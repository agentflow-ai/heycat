//! Common utilities for Tauri commands.
//!
//! This module provides shared infrastructure for commands:
//! - `emitter`: TauriEventEmitter for production event emission
//! - `state_access`: Unified settings access helper

mod emitter;
mod state_access;

pub use emitter::TauriEventEmitter;
pub use state_access::{get_settings_file, SettingsHelper};

/// Helper macro to emit events with error logging.
///
/// Logs a warning if the event emission fails instead of propagating the error.
///
/// # Example
/// ```ignore
/// emit_or_warn!(app_handle, "event_name", payload);
/// ```
#[macro_export]
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            crate::warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
