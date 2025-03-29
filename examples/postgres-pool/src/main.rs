//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace TRACE_LOG_LEVEL=trace cargo run -p example-postgres-pool
//! ```

use std::env;

use axum::{extract::State, http::StatusCode};
use caslex_extra::{cleanup_resources, setup_application, storages::postgres_pool};
use caslex_http::server::{Config, Server};
use utoipa_axum::{router::OpenApiRouter, routes};

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    setup_application(SERVICE_NAME);

    let pg_config = postgres_pool::Config::parse();
    let pool = postgres_pool::build_pool_from_config(pg_config)
        .await
        .unwrap();

    let router = OpenApiRouter::new()
        .routes(routes!(handler))
        .with_state(pool.clone());

    let server_config = Config::parse();
    let result = Server::new(server_config).router(router).run().await;

    cleanup_resources();

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("failed to start server: {}", e);
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
async fn handler(
    State(state): State<deadpool_postgres::Pool>,
) -> Result<String, (StatusCode, String)> {
    let conn = state.get().await.map_err(internal_error)?;

    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;
    let two: i32 = row.try_get(0).map_err(internal_error)?;

    Ok(two.to_string())
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
