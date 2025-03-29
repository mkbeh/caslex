use std::{collections::HashMap, error::Error as StdError, fmt, fmt::Display, sync::LazyLock};

use axum_core::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use caslex_extra::security::jwt;
use http::{StatusCode, request::Parts};
use jsonwebtoken::errors::ErrorKind;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, DefaultError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = DefaultError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let derr = DefaultError::AppError;

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| derr(&AuthError::InvalidToken))?;

        let token_data = match jwt::decode_token::<Claims>(bearer.token()) {
            Ok(data) => data,
            Err(err) => match err.kind() {
                ErrorKind::ExpiredSignature => Err(derr(&AuthError::ExpiredSignature))?,
                ErrorKind::InvalidToken => Err(derr(&AuthError::InvalidToken))?,
                ErrorKind::InvalidSignature => Err(derr(&AuthError::InvalidSignature))?,
                ErrorKind::Json(_) => Err(derr(&AuthError::InvalidClaims))?,
                _ => Err(derr(&AuthError::InvalidToken))?,
            },
        };

        Ok(token_data.claims)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidSignature,
    InvalidClaims,
    ExpiredSignature,
}

impl StdError for AuthError {}

impl Display for AuthError {
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

impl AppError for AuthError {
    fn status(&self) -> StatusCode {
        AUTH_ERRORS
            .get(self)
            .map_or(StatusCode::INTERNAL_SERVER_ERROR, |e| e.code)
    }

    fn details(&self) -> String {
        AUTH_ERRORS
            .get(self)
            .map_or("unknown error".to_string(), |e| e.details.to_string())
    }

    fn kind(&self) -> String {
        AUTH_ERRORS
            .get(self)
            .map_or("unknown_error".to_string(), |e| e.kind.to_string())
    }
}

struct FullError {
    code: StatusCode,
    kind: String,
    details: String,
}

static AUTH_ERRORS: LazyLock<HashMap<AuthError, FullError>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert(
        AuthError::WrongCredentials,
        FullError {
            code: StatusCode::UNAUTHORIZED,
            kind: "auth_wrong_credentials".to_string(),
            details: "wrong credentials".to_string(),
        },
    );

    map.insert(
        AuthError::MissingCredentials,
        FullError {
            code: StatusCode::BAD_REQUEST,
            kind: "auth_missing_credentials".to_string(),
            details: "missing credentials".to_string(),
        },
    );

    map.insert(
        AuthError::TokenCreation,
        FullError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "auth_token_creation".to_string(),
            details: "token creation".to_string(),
        },
    );

    map.insert(
        AuthError::InvalidToken,
        FullError {
            code: StatusCode::BAD_REQUEST,
            kind: "auth_invalid_token".to_string(),
            details: "invalid token".to_string(),
        },
    );

    map.insert(
        AuthError::InvalidSignature,
        FullError {
            code: StatusCode::UNAUTHORIZED,
            kind: "auth_invalid_signature".to_string(),
            details: "invalid signature".to_string(),
        },
    );

    map.insert(
        AuthError::InvalidClaims,
        FullError {
            code: StatusCode::UNAUTHORIZED,
            kind: "auth_invalid_claims".to_string(),
            details: "invalid claims".to_string(),
        },
    );

    map.insert(
        AuthError::ExpiredSignature,
        FullError {
            code: StatusCode::UNAUTHORIZED,
            kind: "auth_expired_signature".to_string(),
            details: "expired signature".to_string(),
        },
    );

    map
});
