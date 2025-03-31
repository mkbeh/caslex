//! caslex is a set of tools for creating web services.
//!
//! ## High level features
//! * HTTP web server
//! * HTTP middlewares (auth, metrics, trace)
//! * Builtin OpenAPI visualizer
//! * Errors handling
//! * JWT
//! * Postgres Pool
//! * Observability
//! * Extra utils
//!
//! ## Example
//!
//! The “Hello, World!” of caslex is:
//!
//! ```rust,no_run
//! use caslex::server::{Config, Server};
//! use utoipa_axum::{router::OpenApiRouter, routes};
//!
//! #[utoipa::path(
//!     get,
//!     path = "/",
//!     responses(
//!         (status = 200, description = "Ok")
//!     )
//! )]
//! async fn handler() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::parse();
//!     let router = OpenApiRouter::new().routes(routes!(handler));
//!
//!     let result = Server::new(config).router(router).run().await;
//!     match result {
//!         Ok(_) => std::process::exit(0),
//!         Err(_) => {
//!             std::process::exit(1);
//!         }
//!     }
//! }
//! ```
//!
//! Note using `#[tokio::main]` requires you enable tokio’s `macros` and `rt-multi-thread` features
//! or just `full` to enable all features (`cargo add tokio --features macros,rt-multi-thread`).
//!
//! # Errors
//!
//! Example of how to create custom API error.
//!
//! ```rust,no_run
//! use std::{error::Error as StdError, fmt, fmt::Display};
//!
//! use caslex::{
//!     errors::{AppError, DefaultError},
//!     server::{Config, Server},
//! };
//! use http::StatusCode;
//! use utoipa_axum::{router::OpenApiRouter, routes};
//!
//! async fn error_handler() -> Result<&'static str, DefaultError> {
//!     Err(DefaultError::AppError(&CustomError::TestErrorOne))
//! }
//!
//! #[derive(Debug)]
//! enum CustomError {
//!     TestErrorOne,
//! }
//!
//! impl StdError for CustomError {}
//!
//! impl Display for CustomError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(
//!             f,
//!             "error: status={} kind={} details={}",
//!             self.status(),
//!             self.kind(),
//!             self.details()
//!         )
//!     }
//! }
//!
//! impl AppError for CustomError {
//!     fn status(&self) -> StatusCode {
//!         match self {
//!             CustomError::TestErrorOne => StatusCode::BAD_REQUEST,
//!         }
//!     }
//!
//!     fn details(&self) -> String {
//!         match self {
//!             CustomError::TestErrorOne => "my test error".to_owned(),
//!         }
//!     }
//!
//!     fn kind(&self) -> String {
//!         match self {
//!             CustomError::TestErrorOne => "test_error".to_owned(),
//!         }
//!     }
//! }
//! ```
//!
//! # Middlewares
//!
//! ```rust,no_run
//! use caslex::{errors::DefaultError, middlewares::auth::Claims};
//!
//! async fn decode_handler(_: Claims) {
//!     // will be error before enter the body
//! }
//! ```
//!
//! # Examples
//!
//! The caslex repo contains a number of [examples] that show how to put all the pieces together.
//!
//! # Feature flags
//!
//! caslex uses a set of [feature flags] to reduce the amount of compiled and
//! optional dependencies.
//!
//! The following optional features are available:
//!
//! Name | Description | Default?
//! ---|---|---
//! `auth` | Enables auth middleware | No
//!
//! [feature flags]: https://doc.rust-lang.org/cargo/reference/features.html#the-features-section
//! [examples]: https://github.com/mkbeh/caslex/tree/main/examples

mod extractors;
mod metrics;
mod swagger;
mod trace;

pub mod errors;
pub mod middlewares;
pub mod server;
