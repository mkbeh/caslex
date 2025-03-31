//! Contains custom hooks.
//!
//! Replace error exit code to 1 and use tracing to show panic error message.
//!
//! # Example
//!
//! ```rust,no_run
//! use caslex_extra::hooks::setup_panic_hook;
//!
//! setup_panic_hook();
//! panic!("test")
//! ```

/// Setup custom panic hook.
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(move |panic_info| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic_info.location() {
            tracing::error!(
                message = %panic_info,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic_info);
        }
        std::process::exit(1);
    }))
}
