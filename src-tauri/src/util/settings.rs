//! Unified settings access utilities.
//!
//! Provides a consistent way to access settings across the codebase,
//! eliminating duplicated patterns for worktree-aware settings file access.

use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

/// Get the settings file name for the current worktree context.
///
/// Falls back to "settings.json" if worktree state is not available.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
///
/// # Returns
/// The settings file name, e.g., "settings.json" or "settings-worktree-name.json"
pub fn get_settings_file(app_handle: &AppHandle) -> String {
    app_handle
        .try_state::<crate::worktree::WorktreeState>()
        .map(|s| s.settings_file_name())
        .unwrap_or_else(|| crate::worktree::DEFAULT_SETTINGS_FILE.to_string())
}

/// Trait for unified settings access on types that can provide an AppHandle.
///
/// Implementations can use this trait to provide type-safe get/set operations
/// for settings values.
pub trait SettingsAccess {
    /// Get the associated AppHandle.
    fn app_handle(&self) -> Option<&AppHandle>;

    /// Get the settings file name for the current context.
    fn settings_file_name(&self) -> String {
        self.app_handle()
            .map(get_settings_file)
            .unwrap_or_else(|| crate::worktree::DEFAULT_SETTINGS_FILE.to_string())
    }

    /// Get a setting value by key.
    ///
    /// # Arguments
    /// * `key` - The dot-notation key path (e.g., "audio.selectedDevice")
    ///
    /// # Returns
    /// The value as a string if found, None otherwise.
    fn get_setting(&self, key: &str) -> Option<String> {
        let app = self.app_handle()?;
        let settings_file = self.settings_file_name();
        app.store(&settings_file)
            .ok()
            .and_then(|store| store.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

}

#[cfg(test)]
#[path = "settings_test.rs"]
mod tests;
