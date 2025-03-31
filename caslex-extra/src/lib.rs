//! Extra tools for creating web services.
//!
//! # Examples
//!
//! The caslex repo contains a number of [examples] that show how to put all the pieces together.
//!
//! # Feature flags
//!
//! caslex uses a set of [feature flags] to reduce the amount of compiled and
//! optional dependencies.
//!
//! The following optional features are available:
//!
//! Name | Description | Default?
//! ---|---|---
//! `jwt` | Enables jwt supporting | No
//! `postgres` | Enables postgres pool | No
//! `observability` | Enables tracing and logging supporting | No
//!
//! [feature flags]: https://doc.rust-lang.org/cargo/reference/features.html#the-features-section
//! [examples]: https://github.com/mkbeh/caslex/tree/main/examples

#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod closer;
pub mod hooks;
#[cfg(feature = "observability")]
pub mod observability;
pub mod security;
pub mod storages;

pub use closer::cleanup_resources;

/// Setup application defaults such as custom panic hook and opentelemetry.
pub fn setup_application(_name: &'static str) {
    // Setup custom panic hook
    hooks::setup_panic_hook();

    // Setup logs/tracing
    #[cfg(feature = "observability")]
    observability::setup_opentelemetry(_name);
}
