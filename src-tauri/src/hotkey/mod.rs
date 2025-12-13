// Global hotkey registration module

mod tauri_backend;
pub use tauri_backend::TauriShortcutBackend;

pub mod integration;
pub use integration::HotkeyIntegration;

#[cfg(test)]
mod mod_test;

#[cfg(test)]
mod integration_test;

/// The keyboard shortcut for recording (platform-agnostic)
pub const RECORDING_SHORTCUT: &str = "CmdOrControl+Shift+R";

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
pub struct HotkeyService<B: ShortcutBackend> {
    backend: B,
}

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
}
