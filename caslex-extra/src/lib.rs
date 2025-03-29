pub mod closer;
pub mod hooks;
pub mod observability;
pub mod security;
pub mod storages;

pub use closer::cleanup_resources;

pub fn setup_application(name: &'static str) {
    hooks::setup_panic_hook();

    // Setup logs/tracing
    observability::setup_opentelemetry(name.to_string());
    closer::push_callback(Box::new(|| {
        observability::unset_opentelemetry(name.to_owned())
    }));
}
