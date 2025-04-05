//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace OTEL_LOG_LEVEL=trace JWT_SECRET=super-secret-key cargo run -p example-http-jwt-auth
//! ```

#![allow(clippy::exit)]

use std::env;

use axum::Json;
use caslex::{
    errors::DefaultError,
    middlewares::auth::Claims,
    server::{Config, Server},
};
use caslex_extra::{
    observability::{setup_opentelemetry, unset_opentelemetry},
    security::jwt,
};
use serde::Serialize;
use utoipa_axum::{router::OpenApiRouter, routes};

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    setup_opentelemetry(SERVICE_NAME);

    let config = Config::parse();
    let router = OpenApiRouter::new()
        .routes(routes!(encode_handler))
        .routes(routes!(decode_handler));

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

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

#[utoipa::path(
    get,
    path = "/encode_token",
    responses(
        (status = 200, description = "Ok")
    )
)]
async fn encode_handler() -> Result<Json<TokenResponse>, DefaultError> {
    const USER_ID: i32 = 123;
    const TOKEN_LIFETIME_SECS: u64 = 300;

    let claims = Claims {
        sub: USER_ID.to_string(),
        exp: jwt::expiry(TOKEN_LIFETIME_SECS),
    };

    let token = jwt::encode_token(&claims);

    Ok(Json(TokenResponse {
        token: token.unwrap(),
    }))
}

#[utoipa::path(
    get,
    path = "/decode_token",
    responses(
        (status = 200, description = "Ok")
    )
)]
async fn decode_handler(claims: Claims) -> Result<Json<Claims>, DefaultError> {
    Ok(Json(claims))
}
