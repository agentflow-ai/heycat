//! Unified settings access for commands.
//!
//! Provides helpers to access settings from commands in a consistent way,
//! handling worktree context.

use tauri::AppHandle;

/// Get the settings file name for the current worktree context.
///
/// This is a convenience function that wraps the util module's get_settings_file.
/// Falls back to "settings.json" if worktree state is not available.
pub fn get_settings_file(app_handle: &AppHandle) -> String {
    crate::util::get_settings_file(app_handle)
}

#[cfg(test)]
#[path = "state_access_test.rs"]
mod tests;
