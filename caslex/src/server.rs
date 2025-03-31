use std::{
    any::Any, borrow::Cow, fmt::Display, iter::once, net::SocketAddr, sync::LazyLock,
    time::Duration,
};

use anyhow::anyhow;
use async_trait::async_trait;
use axum::{
    Router,
    http::{HeaderName, StatusCode, header::AUTHORIZATION},
    middleware,
    routing::get,
};
use axum_core::response::Response;
use bytes::Bytes;
use clap::Parser;
use http::header;
use http_body_util::Full;
use tokio::{signal, time::timeout};
use tokio_util::sync::CancellationToken;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    propagate_header::PropagateHeaderLayer, sensitive_headers::SetSensitiveRequestHeadersLayer,
    timeout::TimeoutLayer,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    errors::{ErrorInfo, ErrorResponse},
    middlewares, swagger,
};

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "9000")]
    port: String,
    #[arg(long, env = "SERVER_METRICS_PORT", default_value = "9007")]
    metrics_port: String,
    #[arg(long, env = "SERVER_REQUEST_TIMEOUT", default_value = "10s")]
    request_timeout: humantime::Duration,
    #[arg(long, env = "SERVER_DOCS_URL", default_value = "/docs")]
    docs_url: String,
}

impl Config {
    pub fn parse() -> Config {
        Config::try_parse().unwrap()
    }

    fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn get_metrics_addr(&self) -> String {
        format!("{}:{}", self.host, self.metrics_port)
    }
}

#[async_trait]
pub trait Process: Send + Sync {
    async fn pre_run(&self) -> anyhow::Result<()>;
    async fn run(&self, token: CancellationToken) -> anyhow::Result<()>;
}

pub struct Server<'a> {
    addr: String,
    metrics_addr: String,
    request_timeout: Duration,
    docs_url: String,
    router: Option<OpenApiRouter>,
    processes: Option<&'a Vec<&'static dyn Process>>,
}

macro_rules! server_method {
    ($name:ident, $ty:ty) => {
        pub fn $name(mut self, $name: $ty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

enum ServerKind {
    Application,
    Metrics,
}

impl Display for ServerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Application => write!(f, "application"),
            Self::Metrics => write!(f, "metrics"),
        }
    }
}

impl<'a> Server<'a> {
    server_method!(router, OpenApiRouter);
    server_method!(processes, &'a Vec<&'static dyn Process>);

    pub fn new(cfg: Config) -> Self {
        Server {
            addr: cfg.get_addr(),
            metrics_addr: cfg.get_metrics_addr(),
            request_timeout: cfg.request_timeout.into(),
            docs_url: cfg.docs_url,
            router: None,
            processes: None,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        const PROCESS_PRE_RUN_TIMEOUT: Duration = Duration::from_secs(60);
        static SHUTDOWN_TOKEN: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);

        let app_server = self.bootstrap_server(
            self.addr.clone(),
            self.setup_router(),
            ServerKind::Application,
        );

        let metrics_server = self.bootstrap_server(
            self.metrics_addr.clone(),
            get_metrics_router(),
            ServerKind::Metrics,
        );

        let processes = match self.processes {
            Some(processes) => processes,
            _ => &vec![],
        };

        // pre run processes
        {
            let tasks: Vec<_> = processes
                .iter()
                .map(|p| {
                    tokio::spawn(timeout(PROCESS_PRE_RUN_TIMEOUT, async {
                        p.pre_run().await
                    }))
                })
                .collect();

            for task in tasks {
                if let Err(e) = task.await? {
                    return Err(anyhow!("error while pre run process: {}", e));
                }
            }
        }

        // disable failure in the custom panic hook when there is a panic,
        // because we can't handle the panic in the panic middleware (exit(1) trouble)
        setup_panic_hook();

        {
            // run processes
            let runnable_tasks: Vec<_> = processes
                .iter()
                .map(|p| tokio::spawn(async { p.run(SHUTDOWN_TOKEN.clone()).await }))
                .collect();

            tokio::try_join!(app_server, metrics_server)
                .map_err(|e| anyhow!("Failed to bootstrap server. Reason: {:?}", e))?;

            SHUTDOWN_TOKEN.cancel();

            for task in runnable_tasks {
                if let Err(e) = task.await? {
                    tracing::error!("Failed to shutdown processes. Reason: {:?}", e);
                }
            }
        }

        Ok(())
    }

    async fn bootstrap_server(
        &self,
        addr: String,
        router: Router,
        server_kind: ServerKind,
    ) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(addr.clone())
            .await
            .map_err(|e| anyhow!("failed to bind to address: {e}"))?;

        tracing::info!("listening {server_kind} server on {addr}");

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow!("failed to start server on address {addr}: {e}"))?;

        Ok(())
    }

    fn setup_router(&self) -> Router {
        let _router = match self.router.clone() {
            Some(router) => router.merge(get_default_router()),
            _ => get_default_router(),
        };

        middlewares::with_trace_layer(swagger::get_openapi_router(_router, self.docs_url.clone()))
            // Fallback 404
            .fallback(fallback_handler)
            // Fallback 405
            .method_not_allowed_fallback(fallback_handler_405)
            // Panic recovery handler
            .layer(CatchPanicLayer::custom(panic_handler))
            // Prometheus metrics tracker
            .layer(middleware::from_fn(middlewares::metrics_handler))
            // Request timeout
            .layer(TimeoutLayer::new(self.request_timeout))
            // Compress responses
            .layer(CompressionLayer::new())
            // Mark the `Authorization` request header as sensitive so it doesn't show in logs
            .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
            // Propagate headers from requests to responses
            .layer(PropagateHeaderLayer::new(HeaderName::from_static(
                "x-request-id",
            )))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    let quit = async {
        signal::unix::signal(signal::unix::SignalKind::quit())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = quit => {},
    }
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(move |panic_info| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic_info.location() {
            tracing::error!(
                message = %panic_info,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic_info);
        }
    }))
}

fn get_default_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(readiness))
        .routes(routes!(liveness))
}

fn get_metrics_router() -> Router {
    Router::from(get_default_router()).route("/metrics", get(middlewares::prometheus_handler))
}

/// readiness
#[utoipa::path(
    get,
    path = "/readiness",
    tag = "health",
    responses(
        (status = 200)
    )
)]
async fn readiness() -> (StatusCode, Cow<'static, str>) {
    (StatusCode::OK, Cow::from("OK"))
}

/// liveness
#[utoipa::path(
    get,
    path = "/liveness",
    tag = "health",
    responses(
        (status = 200)
    )
)]
async fn liveness() -> (StatusCode, Cow<'static, str>) {
    (StatusCode::OK, Cow::from("OK"))
}

fn panic_handler(err: Box<dyn Any + Send + 'static>) -> Response<Full<Bytes>> {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        (*s).to_owned()
    } else {
        "Unknown panic message".to_owned()
    };

    let body = ErrorResponse {
        error: ErrorInfo {
            kind: "unhandled_error".to_owned(),
            details,
        },
    };
    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::from(body))
        .unwrap()
}

async fn fallback_handler() -> Response<Full<Bytes>> {
    let body = ErrorResponse {
        error: ErrorInfo {
            kind: "method_not_found".to_owned(),
            details: "method not found".to_owned(),
        },
    };
    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::from(body))
        .unwrap()
}

async fn fallback_handler_405() -> Response<Full<Bytes>> {
    let body = ErrorResponse {
        error: ErrorInfo {
            kind: "method_not_allowed".to_owned(),
            details: "method not allowed".to_owned(),
        },
    };
    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::from(body))
        .unwrap()
}
