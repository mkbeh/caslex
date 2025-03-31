//! Extra tools for creating web services.

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
