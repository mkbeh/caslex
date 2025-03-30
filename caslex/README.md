# caslex

`caslex` is a set of tools for creating web services.

[![Latest Version](https://img.shields.io/crates/v/caslex.svg)](https://crates.io/crates/caslex)

More information about this crate can be found in the [crate documentation](https://docs.rs/caslex/latest/caslex/).

## High level features

* HTTP web server
* HTTP middlewares (auth, metrics, trace)
* Builtin OpenAPI visualizer
* Errors handling
* JWT
* Postgres Pool
* Observability
* Extra utils

## Usage example

```rust
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
```

You can find this [example](https://github.com/mkbeh/caslex/tree/main/examples/http-hello-world) as well as other example projects in the [example directory](https://github.com/mkbeh/caslex/tree/main/examples).

See the [crate documentation](https://docs.rs/caslex/latest/caslex/) for way more examples.

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## Examples

The [examples](https://github.com/mkbeh/caslex/tree/main/examples) folder contains various examples of how to use axum. The [docs](https://docs.rs/caslex/latest/caslex/) also provide lots of code snippets and examples.

## Projects

List of projects using `caslex`:

- [rust-simple-chat](https://github.com/mkbeh/rust-simple-chat): sample project.

## License

This project is licensed under the [MIT license](https://github.com/mkbeh/caslex/tree/main/caslex/LICENSE).