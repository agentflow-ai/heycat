//! Unified settings access for commands.
//!
//! Provides helpers to access settings from commands in a consistent way,
//! handling worktree context and providing common settings accessors.

use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::util::SettingsAccess;

/// Get the settings file name for the current worktree context.
///
/// This is a convenience function that wraps the util module's get_settings_file.
/// Falls back to "settings.json" if worktree state is not available.
pub fn get_settings_file(app_handle: &AppHandle) -> String {
    crate::util::get_settings_file(app_handle)
}

/// Helper for settings access in commands.
///
/// Provides convenient methods for accessing common settings values
/// used by commands, wrapping the SettingsAccess trait.
pub struct SettingsHelper<'a> {
    app_handle: &'a AppHandle,
}

impl<'a> SettingsHelper<'a> {
    /// Create a new SettingsHelper with the given AppHandle.
    pub fn new(app_handle: &'a AppHandle) -> Self {
        Self { app_handle }
    }

    /// Get the recording shortcut from settings.
    pub fn get_recording_shortcut(&self) -> Option<String> {
        self.get_setting("hotkey.recordingShortcut")
    }

    /// Get the recording mode from settings.
    pub fn get_recording_mode(&self) -> crate::hotkey::RecordingMode {
        self.get_setting_json("shortcuts.recordingMode")
            .unwrap_or_default()
    }

    /// Get the selected audio device from settings.
    pub fn get_selected_audio_device(&self) -> Option<String> {
        self.get_setting("audio.selectedDevice")
    }
}

impl<'a> SettingsAccess for SettingsHelper<'a> {
    fn app_handle(&self) -> Option<&AppHandle> {
        Some(self.app_handle)
    }
}

/// Extension trait for AppHandle to provide direct settings access.
///
/// This allows using settings methods directly on AppHandle without
/// needing to create a SettingsHelper.
pub trait AppHandleSettingsExt {
    /// Get the settings file name for the current worktree context.
    fn settings_file_name(&self) -> String;

    /// Get a setting value by key.
    fn get_setting(&self, key: &str) -> Option<String>;

    /// Get a setting value with JSON deserialization.
    fn get_setting_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// Set a setting value by key.
    fn set_setting<T: serde::Serialize>(&self, key: &str, value: T) -> Result<(), String>;
}

impl AppHandleSettingsExt for AppHandle {
    fn settings_file_name(&self) -> String {
        get_settings_file(self)
    }

    fn get_setting(&self, key: &str) -> Option<String> {
        let settings_file = self.settings_file_name();
        self.store(&settings_file)
            .ok()
            .and_then(|store| store.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    fn get_setting_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let settings_file = self.settings_file_name();
        self.store(&settings_file)
            .ok()
            .and_then(|store| store.get(key))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    fn set_setting<T: serde::Serialize>(&self, key: &str, value: T) -> Result<(), String> {
        let settings_file = self.settings_file_name();
        let store = self
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
#[path = "state_access_test.rs"]
mod tests;
