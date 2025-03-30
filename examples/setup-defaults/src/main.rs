//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace TRACE_LOG_LEVEL=trace cargo run -p example-setup-defaults
//! ```

#![allow(clippy::exit)]

use std::env;

use caslex_extra::{cleanup_resources, setup_application};
use caslex::server::{Config, Server};
use utoipa_axum::{router::OpenApiRouter, routes};

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    // Setup application defaults.
    setup_application(SERVICE_NAME);

    let config = Config::parse();
    let router = OpenApiRouter::new().routes(routes!(handler));

    let result = Server::new(config).router(router).run().await;

    // Cleanup application resources.
    cleanup_resources();

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
