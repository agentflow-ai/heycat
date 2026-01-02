// Global hotkey registration module

use serde::{Deserialize, Serialize};

mod tauri_backend;
// TauriShortcutBackend is used internally by create_shortcut_backend on non-macOS platforms
#[allow(unused_imports)]
pub use tauri_backend::TauriShortcutBackend;

#[cfg(target_os = "macos")]
pub mod cgeventtap_backend;

#[cfg(not(target_os = "macos"))]
mod rdev_backend;
#[cfg(not(target_os = "macos"))]
pub use rdev_backend::RdevShortcutBackend;

pub mod double_tap;

#[cfg(test)]
mod escape;

pub mod integration;
pub use integration::HotkeyIntegration;

#[cfg(test)]
mod mod_test;

#[cfg(test)]
mod integration_test;

use std::sync::Arc;

/// The keyboard shortcut for cancel (Escape key)
pub const ESCAPE_SHORTCUT: &str = "Escape";

/// Errors that can occur during hotkey registration
#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyError {
    /// Failed to register the shortcut
    RegistrationFailed(String),
    /// The shortcut is already registered
    AlreadyRegistered,
    /// The shortcut conflicts with another application
    Conflict(String),
}

impl std::fmt::Display for HotkeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotkeyError::RegistrationFailed(msg) => write!(f, "Registration failed: {}", msg),
            HotkeyError::AlreadyRegistered => write!(f, "Shortcut already registered"),
            HotkeyError::Conflict(msg) => write!(f, "Shortcut conflict: {}", msg),
        }
    }
}

impl std::error::Error for HotkeyError {}

/// Recording mode determines how the hotkey triggers recording
///
/// Used by HotkeyIntegration and settings commands to support push-to-talk mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RecordingMode {
    /// Press once to start, press again to stop (default)
    #[default]
    Toggle,
    /// Hold to record, release to stop
    PushToTalk,
}

/// Map backend error messages to HotkeyError variants
pub fn map_backend_error(msg: &str) -> HotkeyError {
    let lower = msg.to_lowercase();
    if lower.contains("already registered") {
        HotkeyError::AlreadyRegistered
    } else if lower.contains("conflict") || lower.contains("in use") {
        HotkeyError::Conflict(msg.to_string())
    } else {
        HotkeyError::RegistrationFailed(msg.to_string())
    }
}

/// Trait for shortcut registration backends (allows mocking in tests)
pub trait ShortcutBackend {
    fn register(&self, shortcut: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String>;
    fn unregister(&self, shortcut: &str) -> Result<(), String>;

    /// Returns a reference to Any for downcasting to concrete types
    ///
    /// This enables checking if a backend implements ShortcutBackendExt
    /// for push-to-talk mode support.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Extended trait for shortcut backends that support key release detection
///
/// This trait adds support for push-to-talk mode where we need separate
/// callbacks for key press and key release events.
///
/// Implemented by CGEventTapHotkeyBackend (macOS) and RdevShortcutBackend (Windows/Linux).
pub trait ShortcutBackendExt: ShortcutBackend {
    /// Register a shortcut with separate press and release callbacks
    ///
    /// - `on_press`: Called when the key is pressed down
    /// - `on_release`: Called when the key is released
    ///
    /// Returns an error if registration fails.
    fn register_with_release(
        &self,
        shortcut: &str,
        on_press: Box<dyn Fn() + Send + Sync>,
        on_release: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String>;
}

/// Null implementation of ShortcutBackend for placeholder configs
///
/// This is used when building escape key configuration incrementally
/// via the builder pattern. Registration always fails gracefully.
pub struct NullShortcutBackend;

impl ShortcutBackend for NullShortcutBackend {
    fn register(&self, _shortcut: &str, _callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        Err("NullShortcutBackend: registration not supported".to_string())
    }

    fn unregister(&self, _shortcut: &str) -> Result<(), String> {
        Err("NullShortcutBackend: unregistration not supported".to_string())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Service for managing hotkey registration
///
/// Note: Production code uses HotkeyServiceDyn. This generic version is kept for testing
/// with MockBackend. The #[allow(dead_code)] silences warnings on platforms where it's unused.
#[allow(dead_code)]
pub struct HotkeyService<B: ShortcutBackend> {
    /// The backend used for shortcut registration
    pub backend: B,
}

#[allow(dead_code)]
impl<B: ShortcutBackend> HotkeyService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Register the Escape key shortcut for cancellation
    ///
    /// The Escape key listener should only be active during recording
    /// to avoid conflicts with Escape key usage in other contexts.
    ///
    /// Note: Currently unused as HotkeyIntegration calls backend.register() directly,
    /// but kept as public API for potential future use.
    #[allow(dead_code)]
    pub fn register_escape_shortcut(
        &self,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), HotkeyError> {
        self.backend
            .register(ESCAPE_SHORTCUT, callback)
            .map_err(|e| map_backend_error(&e))
    }

    /// Unregister the Escape key shortcut
    ///
    /// Note: Currently unused as HotkeyIntegration calls backend.unregister() directly,
    /// but kept as public API for potential future use.
    #[allow(dead_code)]
    pub fn unregister_escape_shortcut(&self) -> Result<(), HotkeyError> {
        self.backend
            .unregister(ESCAPE_SHORTCUT)
            .map_err(|e| map_backend_error(&e))
    }
}

/// Service for managing hotkey registration with dynamic backend
///
/// This version uses Arc<dyn ShortcutBackend> for runtime polymorphism,
/// allowing the backend to be selected at runtime based on platform.
pub struct HotkeyServiceDyn {
    /// The backend used for shortcut registration
    pub backend: Arc<dyn ShortcutBackend + Send + Sync>,
}

impl HotkeyServiceDyn {
    pub fn new(backend: Arc<dyn ShortcutBackend + Send + Sync>) -> Self {
        Self { backend }
    }
}

/// Create the appropriate shortcut backend for the current platform
///
/// - **macOS**: Uses CGEventTapHotkeyBackend (supports fn key, media keys, requires Accessibility permission)
/// - **Windows/Linux**: Uses RdevShortcutBackend (supports push-to-talk with key release detection)
///
/// Both backends implement the `ShortcutBackend` trait and `ShortcutBackendExt` for PTT support.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn create_shortcut_backend(app: tauri::AppHandle) -> Arc<dyn ShortcutBackend + Send + Sync> {
    #[cfg(target_os = "macos")]
    {
        // Suppress unused variable warning on macOS since we don't need the app handle
        let _ = app;
        Arc::new(cgeventtap_backend::CGEventTapHotkeyBackend::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Suppress unused variable warning since RdevShortcutBackend doesn't need the app handle
        let _ = app;
        Arc::new(RdevShortcutBackend::new())
    }
}
