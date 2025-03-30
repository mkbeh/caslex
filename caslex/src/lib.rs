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

mod extractors;
mod swagger;

pub mod errors;
pub mod middlewares;
pub mod server;
