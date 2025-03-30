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

#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(rename = "error")]
    pub error: ErrorInfo,
}

#[derive(Serialize)]
pub struct ErrorInfo {
    pub kind: String,
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
