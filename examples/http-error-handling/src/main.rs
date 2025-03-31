//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace TRACE_LOG_LEVEL=trace cargo run -p example-http-error-handling
//! ```

#![allow(clippy::exit)]

use std::{env, error::Error as StdError, fmt, fmt::Display};

use anyhow::anyhow;
use axum::http::StatusCode;
use caslex::{
    errors::{AppError, AppJson, DefaultError},
    server::{Config, Server},
};
use caslex_extra::observability::{setup_opentelemetry, unset_opentelemetry};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    setup_opentelemetry(SERVICE_NAME);

    let config = Config::parse();
    let router = OpenApiRouter::new()
        .routes(routes!(custom_error_handler))
        .routes(routes!(validation_error_handler))
        .routes(routes!(other_error_handler));

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
    post,
    path = "/validation",
    request_body = BodyError,
    responses(
        (status = 400, description = "returns validation error")
    )
)]
async fn validation_error_handler(
    AppJson(payload): AppJson<BodyError>,
) -> Result<&'static str, DefaultError> {
    match payload.validate() {
        Ok(_) => Ok("nothing"),
        Err(err) => Err(DefaultError::ValidationError(err)),
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct BodyError {
    #[validate(length(min = 1, max = 300))]
    message: String,
}

#[utoipa::path(
    post,
    path = "/other",
    responses(
        (status = 400, description = "returns validation error")
    )
)]
async fn other_error_handler() -> Result<&'static str, DefaultError> {
    Err(DefaultError::Other(anyhow!("other error")))
}

#[utoipa::path(
    get,
    path = "/custom",
    responses(
        (status = 400, description = "returns custom error")
    )
)]
async fn custom_error_handler() -> Result<&'static str, DefaultError> {
    Err(DefaultError::AppError(&CustomError::TestErrorOne))
}

#[derive(Debug)]
enum CustomError {
    TestErrorOne,
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
        }
    }

    fn details(&self) -> String {
        match self {
            CustomError::TestErrorOne => "test error one".to_owned(),
        }
    }

    fn kind(&self) -> String {
        match self {
            CustomError::TestErrorOne => "test_error_one".to_owned(),
        }
    }
}
