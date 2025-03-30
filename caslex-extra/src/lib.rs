//! TODO docs

pub mod closer;
pub mod hooks;
pub mod security;
pub mod storages;

#[cfg(feature = "observability")]
pub mod observability;

pub use closer::cleanup_resources;

pub fn setup_application(_name: &'static str) {
    // Setup custom panic hook
    hooks::setup_panic_hook();

    // Setup logs/tracing
    #[cfg(feature = "observability")]
    observability::setup_opentelemetry(_name);
}
