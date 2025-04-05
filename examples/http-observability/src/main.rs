//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace OTEL_LOG_LEVEL=trace cargo run -p example-observability
//! ```

#![allow(clippy::exit)]

use std::env;

use caslex::server::{Config, Server};
use caslex_extra::observability::{setup_opentelemetry, unset_opentelemetry};
use utoipa_axum::{router::OpenApiRouter, routes};

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    // Logs and tracing visibility configure via env variables
    // such as LOG_LEVEL and OTEL_LOG_LEVEL.

    // init tracing/logging
    setup_opentelemetry(SERVICE_NAME);

    let config = Config::parse();
    let router = OpenApiRouter::new().routes(routes!(handler));

    let result = Server::new(config).router(router).run().await;

    // shutdown tracing/logging
    unset_opentelemetry(SERVICE_NAME);

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("failed to start server: {e}");
            std::process::exit(1);
        }
    }
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Ok")
    )
)]
async fn handler() -> &'static str {
    "Hello, World!"
}
