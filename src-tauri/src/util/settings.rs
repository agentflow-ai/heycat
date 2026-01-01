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

    /// Get a setting value with JSON deserialization.
    ///
    /// # Arguments
    /// * `key` - The dot-notation key path
    ///
    /// # Returns
    /// The deserialized value if found and valid, None otherwise.
    fn get_setting_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let app = self.app_handle()?;
        let settings_file = self.settings_file_name();
        app.store(&settings_file)
            .ok()
            .and_then(|store| store.get(key))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a setting value by key.
    ///
    /// # Arguments
    /// * `key` - The dot-notation key path
    /// * `value` - The value to set (will be serialized to JSON)
    ///
    /// # Returns
    /// Ok(()) on success, Err with message on failure.
    fn set_setting<T: serde::Serialize>(&self, key: &str, value: T) -> Result<(), String> {
        let app = self.app_handle().ok_or("No app handle available")?;
        let settings_file = self.settings_file_name();
        let store = app
            .store(&settings_file)
            .map_err(|e| format!("Failed to open settings store: {}", e))?;

        store.set(
            key,
            serde_json::to_value(value).map_err(|e| format!("Failed to serialize value: {}", e))?,
        );

        store
            .save()
            .map_err(|e| format!("Failed to save settings: {}", e))
    }
}

#[cfg(test)]
#[path = "settings_test.rs"]
mod tests;
