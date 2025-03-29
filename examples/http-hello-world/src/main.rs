//! Run with
//!
//! ```not_rust
//! cargo run -p example-http-hello-world
//! ```

use caslex_http::server::{Config, Server};
use utoipa_axum::{router::OpenApiRouter, routes};

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
