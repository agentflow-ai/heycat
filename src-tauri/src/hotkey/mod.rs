// Global hotkey registration module

mod tauri_backend;
// TauriShortcutBackend is used internally by create_shortcut_backend on non-macOS platforms
#[allow(unused_imports)]
pub use tauri_backend::TauriShortcutBackend;

#[cfg(target_os = "macos")]
mod cgeventtap_backend;

pub mod double_tap;

pub mod integration;
pub use integration::HotkeyIntegration;

#[cfg(test)]
mod mod_test;

#[cfg(test)]
mod integration_test;

use std::sync::Arc;

/// The keyboard shortcut for recording (platform-agnostic)
pub const RECORDING_SHORTCUT: &str = "CmdOrControl+Shift+R";

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

    pub fn register_recording_shortcut(
        &self,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), HotkeyError> {
        self.backend
            .register(RECORDING_SHORTCUT, callback)
            .map_err(|e| map_backend_error(&e))
    }

    pub fn unregister_recording_shortcut(&self) -> Result<(), HotkeyError> {
        self.backend
            .unregister(RECORDING_SHORTCUT)
            .map_err(|e| map_backend_error(&e))
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

    pub fn register_recording_shortcut(
        &self,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), HotkeyError> {
        self.backend
            .register(RECORDING_SHORTCUT, callback)
            .map_err(|e| map_backend_error(&e))
    }

    pub fn unregister_recording_shortcut(&self) -> Result<(), HotkeyError> {
        self.backend
            .unregister(RECORDING_SHORTCUT)
            .map_err(|e| map_backend_error(&e))
    }
}

/// Create the appropriate shortcut backend for the current platform
///
/// - **macOS**: Uses CGEventTapHotkeyBackend (supports fn key, media keys, requires Accessibility permission)
/// - **Windows/Linux**: Uses TauriShortcutBackend (standard Tauri global shortcut plugin)
///
/// Both backends implement the `ShortcutBackend` trait, so they can be used interchangeably.
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
        Arc::new(TauriShortcutBackend::new(app))
    }
}
