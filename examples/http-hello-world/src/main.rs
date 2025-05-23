//! Run with
//!
//! ```not_rust
//! cargo run -p example-http-hello-world
//! ```

#![allow(clippy::exit)]

use caslex::server::{Config, Server};
use utoipa_axum::{router::OpenApiRouter, routes};

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

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let router = OpenApiRouter::new().routes(routes!(handler));

    let result = Server::new(config).router(router).run().await;
    match result {
        Ok(_) => std::process::exit(0),
        Err(_) => {
            std::process::exit(1);
        }
    }
}
