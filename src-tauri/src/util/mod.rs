//! Shared utilities for the heycat backend.
//!
//! This module provides common functionality used across the codebase:
//! - `settings`: SettingsAccess trait for unified settings access
//! - `runtime`: Tokio runtime helpers for async-to-sync bridges

mod runtime;
mod settings;

pub use runtime::run_async;
pub use settings::{get_settings_file, SettingsAccess};

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
