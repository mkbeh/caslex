//! Contains custom API errors.
//!
//! **Note:** JSON rejection errors catches automatically.
//!
//! # Custom error
//!
//! Handling custom error example
//!
//! ```rust,no_run
//! use std::{error::Error as StdError, fmt, fmt::Display};
//!
//! use caslex::errors::{AppError, DefaultError};
//! use http::StatusCode;
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
//! # Struct validation error
//!
//! Handling validation error example
//!
//! ```rust,no_run
//! use caslex::errors::{AppJson, DefaultError};
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[derive(Debug, Deserialize, Validate)]
//! struct BodyError {
//!     #[validate(length(min = 1, max = 300))]
//!     message: String,
//! }
//!
//! async fn validation_error_handler(
//!     AppJson(payload): AppJson<BodyError>,
//! ) -> Result<&'static str, DefaultError> {
//!     match payload.validate() {
//!         Ok(_) => Ok("nothing"),
//!         Err(err) => Err(DefaultError::ValidationError(err)),
//!     }
//! }
//! ```
//!
//!
//! # Other errors
//!
//! Handling other error example
//!
//! ```rust,no_run
//! use anyhow::anyhow;
//! use caslex::errors::DefaultError;
//!
//! async fn other_error_handler() -> Result<&'static str, DefaultError> {
//!     Err(DefaultError::Other(anyhow!("other error")))
//! }
//! ```

use std::{error::Error as StdError, fmt::Debug};

use axum::{
    Json,
    extract::{FromRequest, rejection::JsonRejection},
};
use axum_core::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

pub trait AppError: StdError {
    fn status(&self) -> StatusCode;
    fn details(&self) -> String;
    fn kind(&self) -> String;
}

/// Define default custom API error.
#[derive(Error, Debug)]
pub enum DefaultError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    #[error("application error")]
    AppError(#[source] &'static dyn AppError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Define error response.
#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(rename = "error")]
    pub error: ErrorInfo,
}

/// Define error info.
#[derive(Serialize)]
pub struct ErrorInfo {
    /// Error kind.
    pub kind: String,
    /// Error full description.
    pub details: String,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(DefaultError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    for<'a> Json<&'a T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(&self.0).into_response()
    }
}

impl IntoResponse for DefaultError {
    fn into_response(self) -> Response {
        let (status, details, kind) = match self {
            DefaultError::JsonRejection(rejection) => (
                rejection.status(),
                rejection.body_text(),
                "json_rejection".to_owned(),
            ),

            DefaultError::ValidationError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("[{self}]").replace('\n', ", "),
                "validation_error".to_owned(),
            ),

            DefaultError::AppError(application_error) => (
                application_error.status(),
                application_error.details(),
                application_error.kind(),
            ),

            DefaultError::Other(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                "unhandled_error".to_owned(),
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorInfo { kind, details },
        });

        (status, body).into_response()
    }
}
