// CGEventTap-based hotkey backend for macOS
//
// This backend uses CGEventTap to capture keyboard events, enabling support for:
// - fn/Globe key as part of hotkeys
// - Media keys (Play/Pause, Volume, etc.)
// - Modifier-only hotkeys (just fn, or fn+Command)
// - Left/right modifier distinction
//
// This is a macOS-only implementation. Other platforms use TauriShortcutBackend.

use super::{ShortcutBackend, ShortcutBackendExt};
use crate::keyboard_capture::cgeventtap::{CGEventTapCapture, CapturedKeyEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Type alias for the callback map used in CGEventTapHotkeyBackend
type CallbackMap = Arc<Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>>;

/// Media key names that can be used in shortcuts
const MEDIA_KEY_NAMES: &[&str] = &[
    "PlayPause",
    "VolumeUp",
    "VolumeDown",
    "Mute",
    "NextTrack",
    "PreviousTrack",
    "FastForward",
    "Rewind",
    "BrightnessUp",
    "BrightnessDown",
    "KeyboardBrightnessUp",
    "KeyboardBrightnessDown",
];

/// Parsed shortcut specification for matching against key events
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShortcutSpec {
    /// Whether fn key must be pressed
    pub fn_key: bool,
    /// Whether command key must be pressed
    pub command: bool,
    /// Whether control key must be pressed
    pub control: bool,
    /// Whether alt/option key must be pressed
    pub alt: bool,
    /// Whether shift key must be pressed
    pub shift: bool,
    /// The key name (None for modifier-only hotkeys)
    pub key_name: Option<String>,
    /// Whether this is a media key
    pub is_media_key: bool,
}

/// Parse a shortcut string into a ShortcutSpec
///
/// Supports formats:
/// - Standard: "CmdOrControl+Shift+R", "Command+Shift+R", "Escape"
/// - Extended: "fn+Command+R", "Function+Command+R"
/// - Modifier-only: "fn", "Command+Shift"
/// - Media: "PlayPause", "VolumeUp", "Command+PlayPause"
pub fn parse_shortcut(shortcut: &str) -> Result<ShortcutSpec, String> {
    let mut spec = ShortcutSpec::default();

    // Split by + to get modifiers and key
    let parts: Vec<&str> = shortcut.split('+').collect();

    for part in parts {
        let normalized = part.trim();

        match normalized.to_lowercase().as_str() {
            "fn" | "function" => spec.fn_key = true,
            "cmd" | "command" | "cmdorcontrol" | "commandorcontrol" => spec.command = true,
            "ctrl" | "control" => spec.control = true,
            "alt" | "option" => spec.alt = true,
            "shift" => spec.shift = true,
            _ => {
                // Check if it's a media key
                if MEDIA_KEY_NAMES.iter().any(|&mk| mk.eq_ignore_ascii_case(normalized)) {
                    spec.is_media_key = true;
                    // Normalize media key name to match what CGEventTap returns
                    spec.key_name = Some(normalize_media_key_name(normalized));
                } else {
                    // Regular key - normalize to uppercase for letters
                    spec.key_name = Some(normalize_key_name(normalized));
                }
            }
        }
    }

    Ok(spec)
}

/// Normalize a media key name to match CGEventTap output
fn normalize_media_key_name(name: &str) -> String {
    // Match against known media keys case-insensitively
    for &mk in MEDIA_KEY_NAMES {
        if mk.eq_ignore_ascii_case(name) {
            return mk.to_string();
        }
    }
    name.to_string()
}

/// Normalize a regular key name
fn normalize_key_name(name: &str) -> String {
    // Single letter keys should be uppercase
    if name.len() == 1 && name.chars().next().unwrap().is_alphabetic() {
        return name.to_uppercase();
    }

    // Common key name normalizations
    match name.to_lowercase().as_str() {
        "escape" | "esc" => "Escape".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "space" => "Space".to_string(),
        "tab" => "Tab".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" => "Delete".to_string(),
        "up" | "arrowup" => "Up".to_string(),
        "down" | "arrowdown" => "Down".to_string(),
        "left" | "arrowleft" => "Left".to_string(),
        "right" | "arrowright" => "Right".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" => "PageUp".to_string(),
        "pagedown" => "PageDown".to_string(),
        _ => {
            // Check for function keys (F1-F19)
            if name.to_lowercase().starts_with('f') {
                if let Ok(n) = name[1..].parse::<u8>() {
                    if (1..=19).contains(&n) {
                        return format!("F{}", n);
                    }
                }
            }
            // Return as-is for other keys
            name.to_string()
        }
    }
}

/// Check if an event represents a modifier key press that matches the spec
/// This is used for modifier-only shortcuts (e.g., just "fn" or "Command+Shift")
fn is_modifier_key_event(event: &CapturedKeyEvent, spec: &ShortcutSpec) -> bool {
    // The event's key_name tells us what key was actually pressed
    // For modifier-only shortcuts, the pressed key must BE a modifier

    // Check if the key pressed matches the required modifiers
    match event.key_name.as_str() {
        "fn" => spec.fn_key,
        "Command" => spec.command,
        "Control" => spec.control,
        "Alt" => spec.alt,
        "Shift" => spec.shift,
        // CapsLock is technically a modifier but not typically used in shortcuts
        "CapsLock" => false,
        // Any other key is not a modifier
        _ => false,
    }
}

/// Check if a captured key event matches a shortcut spec (press events only)
pub fn matches_shortcut(event: &CapturedKeyEvent, spec: &ShortcutSpec) -> bool {
    // Only match on key press, not release
    if !event.pressed {
        return false;
    }

    matches_shortcut_internal(event, spec)
}

/// Check if a captured key release event matches a shortcut spec
pub fn matches_shortcut_release(event: &CapturedKeyEvent, spec: &ShortcutSpec) -> bool {
    // Only match on key release, not press
    if event.pressed {
        return false;
    }

    matches_shortcut_internal(event, spec)
}

/// Internal matching logic shared between press and release handlers
fn matches_shortcut_internal(event: &CapturedKeyEvent, spec: &ShortcutSpec) -> bool {
    // Check modifiers
    if spec.fn_key != event.fn_key {
        return false;
    }
    if spec.command != event.command {
        return false;
    }
    if spec.control != event.control {
        return false;
    }
    if spec.alt != event.alt {
        return false;
    }
    if spec.shift != event.shift {
        return false;
    }

    // Check key name
    match (&spec.key_name, spec.is_media_key) {
        (None, _) => {
            // Modifier-only shortcut - match when the modifier key itself is pressed
            // We must verify the actual key pressed IS a modifier key, not just any key
            // with modifier flags set (e.g., arrow keys have fn flag set on macOS)
            is_modifier_key_event(event, spec)
        }
        (Some(key_name), true) => {
            // Media key - must be a media key event with matching name
            event.is_media_key && event.key_name.eq_ignore_ascii_case(key_name)
        }
        (Some(key_name), false) => {
            // Regular key - must not be a media key and name must match
            !event.is_media_key && event.key_name.eq_ignore_ascii_case(key_name)
        }
    }
}

/// CGEventTap-based hotkey backend for macOS
///
/// This backend uses CGEventTap to capture keyboard events globally,
/// enabling support for fn key, media keys, and other keys that the
/// standard Tauri global shortcut plugin cannot handle.
pub struct CGEventTapHotkeyBackend {
    /// Capture instance (started lazily when first shortcut registered)
    capture: Arc<Mutex<Option<CGEventTapCapture>>>,
    /// Registered shortcuts mapped to their specs
    registered_shortcuts: Arc<Mutex<HashMap<String, ShortcutSpec>>>,
    /// Callbacks for each registered shortcut (key press)
    callbacks: CallbackMap,
    /// Callbacks for key release (used for push-to-talk mode)
    release_callbacks: CallbackMap,
    /// Whether the event tap is currently running
    running: Arc<AtomicBool>,
}

impl CGEventTapHotkeyBackend {
    /// Create a new CGEventTap hotkey backend
    pub fn new() -> Self {
        Self {
            capture: Arc::new(Mutex::new(None)),
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            release_callbacks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the CGEventTap capture if not already running
    fn start_capture(&self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let shortcuts = self.registered_shortcuts.clone();
        let callbacks = self.callbacks.clone();
        let release_callbacks = self.release_callbacks.clone();

        let mut capture_guard = self.capture.lock().map_err(|e| e.to_string())?;

        let mut capture = CGEventTapCapture::new();

        capture.start(move |event| {
            Self::handle_key_event(&event, &shortcuts, &callbacks, &release_callbacks);
        })?;

        *capture_guard = Some(capture);
        self.running.store(true, Ordering::SeqCst);

        crate::info!("CGEventTapHotkeyBackend: Started capture");
        Ok(())
    }

    /// Stop the CGEventTap capture
    fn stop_capture(&self) -> Result<(), String> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let mut capture_guard = self.capture.lock().map_err(|e| e.to_string())?;

        if let Some(ref mut capture) = *capture_guard {
            capture.stop()?;
        }
        *capture_guard = None;
        self.running.store(false, Ordering::SeqCst);

        crate::info!("CGEventTapHotkeyBackend: Stopped capture");
        Ok(())
    }

    /// Handle a key event from CGEventTap
    fn handle_key_event(
        event: &CapturedKeyEvent,
        shortcuts: &Arc<Mutex<HashMap<String, ShortcutSpec>>>,
        callbacks: &CallbackMap,
        release_callbacks: &CallbackMap,
    ) {
        // Get a snapshot of shortcuts and callbacks to avoid holding locks during callback execution
        let matching_callbacks: Vec<Arc<dyn Fn() + Send + Sync>> = {
            let shortcuts_guard = match shortcuts.lock() {
                Ok(g) => g,
                Err(_) => return,
            };

            // Choose the appropriate callback map and matching function based on press/release
            let callbacks_guard = if event.pressed {
                match callbacks.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                }
            } else {
                match release_callbacks.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                }
            };

            shortcuts_guard
                .iter()
                .filter_map(|(shortcut_str, spec)| {
                    let matches = if event.pressed {
                        matches_shortcut(event, spec)
                    } else {
                        matches_shortcut_release(event, spec)
                    };
                    if matches {
                        callbacks_guard.get(shortcut_str).cloned()
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Execute callbacks outside of lock
        for callback in matching_callbacks {
            callback();
        }
    }
}

impl Default for CGEventTapHotkeyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutBackend for CGEventTapHotkeyBackend {
    fn register(
        &self,
        shortcut: &str,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        // Parse the shortcut
        let spec = parse_shortcut(shortcut)?;

        // Store the shortcut and callback
        {
            let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
            let mut callbacks_guard = self.callbacks.lock().map_err(|e| e.to_string())?;

            if shortcuts_guard.contains_key(shortcut) {
                return Err(format!("Shortcut '{}' is already registered", shortcut));
            }

            shortcuts_guard.insert(shortcut.to_string(), spec);
            callbacks_guard.insert(shortcut.to_string(), Arc::from(callback));
        }

        // Start capture if this is the first registration
        self.start_capture()?;

        crate::info!("CGEventTapHotkeyBackend: Registered shortcut '{}'", shortcut);
        Ok(())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        let should_stop = {
            let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
            let mut callbacks_guard = self.callbacks.lock().map_err(|e| e.to_string())?;
            let mut release_callbacks_guard = self.release_callbacks.lock().map_err(|e| e.to_string())?;

            shortcuts_guard.remove(shortcut);
            callbacks_guard.remove(shortcut);
            release_callbacks_guard.remove(shortcut);

            shortcuts_guard.is_empty()
        };

        // Stop capture if no shortcuts remain
        if should_stop {
            self.stop_capture()?;
        }

        crate::info!(
            "CGEventTapHotkeyBackend: Unregistered shortcut '{}'",
            shortcut
        );
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ShortcutBackendExt for CGEventTapHotkeyBackend {
    fn register_with_release(
        &self,
        shortcut: &str,
        on_press: Box<dyn Fn() + Send + Sync>,
        on_release: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        // Parse the shortcut
        let spec = parse_shortcut(shortcut)?;

        // Store the shortcut and both callbacks
        {
            let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
            let mut callbacks_guard = self.callbacks.lock().map_err(|e| e.to_string())?;
            let mut release_callbacks_guard = self.release_callbacks.lock().map_err(|e| e.to_string())?;

            if shortcuts_guard.contains_key(shortcut) {
                return Err(format!("Shortcut '{}' is already registered", shortcut));
            }

            shortcuts_guard.insert(shortcut.to_string(), spec);
            callbacks_guard.insert(shortcut.to_string(), Arc::from(on_press));
            release_callbacks_guard.insert(shortcut.to_string(), Arc::from(on_release));
        }

        // Start capture if this is the first registration
        self.start_capture()?;

        crate::info!(
            "CGEventTapHotkeyBackend: Registered shortcut '{}' with release callback",
            shortcut
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "cgeventtap_backend_test.rs"]
mod tests;
