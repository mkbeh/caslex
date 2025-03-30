//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace TRACE_LOG_LEVEL=trace cargo run -p example-http-error-handling
//! ```

#![allow(clippy::exit)]

use std::{env, error::Error as StdError, fmt, fmt::Display};

use axum::http::StatusCode;
use caslex::{
    errors::{AppError, DefaultError},
    server::{Config, Server},
};
use caslex_extra::observability::{setup_opentelemetry, unset_opentelemetry};
use utoipa_axum::{router::OpenApiRouter, routes};

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    setup_opentelemetry(SERVICE_NAME);

    let config = Config::parse();
    let router = OpenApiRouter::new()
        .routes(routes!(error_handler_one))
        .routes(routes!(error_handler_two));

    let result = Server::new(config).router(router).run().await;

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
    path = "/one",
    responses(
        (status = 400, description = "returns custom error one")
    )
)]
async fn error_handler_one() -> Result<&'static str, DefaultError> {
    Err(DefaultError::AppError(&CustomError::TestErrorOne))
}

#[utoipa::path(
    get,
    path = "/two",
    responses(
        (status = 500, description = "returns custom error two")
    )
)]
async fn error_handler_two() -> Result<&'static str, DefaultError> {
    Err(DefaultError::AppError(&CustomError::TestErrorTwo))
}

#[derive(Debug)]
enum CustomError {
    TestErrorOne,
    TestErrorTwo,
}

impl StdError for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error: status={} kind={} details={}",
            self.status(),
            self.kind(),
            self.details()
        )
    }
}

impl AppError for CustomError {
    fn status(&self) -> StatusCode {
        match self {
            CustomError::TestErrorOne => StatusCode::BAD_REQUEST,
            CustomError::TestErrorTwo => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn details(&self) -> String {
        match self {
            CustomError::TestErrorOne => "test error one".to_owned(),
            CustomError::TestErrorTwo => "test error two".to_owned(),
        }
    }

    fn kind(&self) -> String {
        match self {
            CustomError::TestErrorOne => "test_error_one".to_owned(),
            CustomError::TestErrorTwo => "test_error_two".to_owned(),
        }
    }
}
