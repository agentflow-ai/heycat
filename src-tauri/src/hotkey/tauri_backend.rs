// Tauri shortcut backend - thin wrapper around tauri_plugin_global_shortcut
use super::ShortcutBackend;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct TauriShortcutBackend {
    app: tauri::AppHandle,
}

impl TauriShortcutBackend {
    /// Create a new TauriShortcutBackend
    ///
    /// Note: On macOS, CGEventTapHotkeyBackend is used instead via create_shortcut_backend().
    /// This is used on Windows/Linux. The #[allow(dead_code)] silences warnings on macOS.
    #[allow(dead_code)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ShortcutBackend for TauriShortcutBackend {
    fn register(&self, shortcut: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        let parsed: Shortcut = shortcut.parse().map_err(|e| format!("{}", e))?;
        self.app
            .global_shortcut()
            .on_shortcut(parsed, move |_, _, event| {
                // Only trigger on key PRESS, not on key RELEASE
                // Without this check, the callback fires twice per keypress
                if event.state == ShortcutState::Pressed {
                    callback()
                }
            })
            .map_err(|e| format!("{}", e))
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        let parsed: Shortcut = shortcut.parse().map_err(|e| format!("{}", e))?;
        self.app
            .global_shortcut()
            .unregister(parsed)
            .map_err(|e| format!("{}", e))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
