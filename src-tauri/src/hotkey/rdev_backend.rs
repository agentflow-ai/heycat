// Rdev-based hotkey backend for Windows/Linux
//
// This backend uses the rdev crate to listen for keyboard events globally,
// supporting both key press and key release events for push-to-talk mode.
// This is used on Windows and Linux. macOS uses CGEventTapHotkeyBackend.

use super::{ShortcutBackend, ShortcutBackendExt};
use rdev::{listen, Event, EventType, Key};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Type alias for callback maps
type CallbackMap = Arc<Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>>;

/// Parse a shortcut string to extract the key
///
/// Supports formats like "Escape", "R", "Command+R", etc.
/// For now, we focus on single key shortcuts since that's the primary use case.
fn parse_shortcut_key(shortcut: &str) -> Result<Key, String> {
    // Split by + and take the last part (the actual key)
    let parts: Vec<&str> = shortcut.split('+').collect();
    let key_name = parts.last().ok_or("Empty shortcut")?.trim();

    // Map key names to rdev Key enum
    match key_name.to_lowercase().as_str() {
        "escape" | "esc" => Ok(Key::Escape),
        "enter" | "return" => Ok(Key::Return),
        "space" => Ok(Key::Space),
        "tab" => Ok(Key::Tab),
        "backspace" => Ok(Key::Backspace),
        "delete" => Ok(Key::Delete),
        "up" | "arrowup" => Ok(Key::UpArrow),
        "down" | "arrowdown" => Ok(Key::DownArrow),
        "left" | "arrowleft" => Ok(Key::LeftArrow),
        "right" | "arrowright" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "a" => Ok(Key::KeyA),
        "b" => Ok(Key::KeyB),
        "c" => Ok(Key::KeyC),
        "d" => Ok(Key::KeyD),
        "e" => Ok(Key::KeyE),
        "f" => Ok(Key::KeyF),
        "g" => Ok(Key::KeyG),
        "h" => Ok(Key::KeyH),
        "i" => Ok(Key::KeyI),
        "j" => Ok(Key::KeyJ),
        "k" => Ok(Key::KeyK),
        "l" => Ok(Key::KeyL),
        "m" => Ok(Key::KeyM),
        "n" => Ok(Key::KeyN),
        "o" => Ok(Key::KeyO),
        "p" => Ok(Key::KeyP),
        "q" => Ok(Key::KeyQ),
        "r" => Ok(Key::KeyR),
        "s" => Ok(Key::KeyS),
        "t" => Ok(Key::KeyT),
        "u" => Ok(Key::KeyU),
        "v" => Ok(Key::KeyV),
        "w" => Ok(Key::KeyW),
        "x" => Ok(Key::KeyX),
        "y" => Ok(Key::KeyY),
        "z" => Ok(Key::KeyZ),
        _ => Err(format!("Unknown key: {}", key_name)),
    }
}

/// Rdev-based shortcut backend for Windows/Linux
///
/// This backend uses rdev to capture keyboard events globally,
/// supporting both key press and key release events for push-to-talk mode.
pub struct RdevShortcutBackend {
    /// Registered shortcuts mapped to their rdev Key
    registered_shortcuts: Arc<Mutex<HashMap<String, Key>>>,
    /// Callbacks for key press events
    press_callbacks: CallbackMap,
    /// Callbacks for key release events
    release_callbacks: CallbackMap,
    /// Whether the listener is running
    running: Arc<AtomicBool>,
    /// Handle to the listener thread
    listener_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RdevShortcutBackend {
    /// Create a new Rdev shortcut backend
    pub fn new() -> Self {
        Self {
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
            press_callbacks: Arc::new(Mutex::new(HashMap::new())),
            release_callbacks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
            listener_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the rdev listener if not already running
    fn start_listener(&self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let shortcuts = self.registered_shortcuts.clone();
        let press_callbacks = self.press_callbacks.clone();
        let release_callbacks = self.release_callbacks.clone();
        let running = self.running.clone();

        running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            let callback = move |event: Event| {
                Self::handle_event(&event, &shortcuts, &press_callbacks, &release_callbacks);
            };

            // rdev::listen blocks until an error occurs
            if let Err(e) = listen(callback) {
                // Log error but don't panic - the listener thread is stopping
                eprintln!("RdevShortcutBackend: Listener error: {:?}", e);
            }

            running.store(false, Ordering::SeqCst);
        });

        let mut handle_guard = self.listener_handle.lock().map_err(|e| e.to_string())?;
        *handle_guard = Some(handle);

        Ok(())
    }

    /// Handle a keyboard event from rdev
    fn handle_event(
        event: &Event,
        shortcuts: &Arc<Mutex<HashMap<String, Key>>>,
        press_callbacks: &CallbackMap,
        release_callbacks: &CallbackMap,
    ) {
        let (key, is_press) = match event.event_type {
            EventType::KeyPress(k) => (k, true),
            EventType::KeyRelease(k) => (k, false),
            _ => return,
        };

        // Find matching shortcuts and collect callbacks
        let matching_callbacks: Vec<Arc<dyn Fn() + Send + Sync>> = {
            let shortcuts_guard = match shortcuts.lock() {
                Ok(g) => g,
                Err(_) => return,
            };

            let callbacks_guard = if is_press {
                match press_callbacks.lock() {
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
                .filter_map(|(shortcut_str, registered_key)| {
                    if *registered_key == key {
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

impl Default for RdevShortcutBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutBackend for RdevShortcutBackend {
    fn register(
        &self,
        shortcut: &str,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        let key = parse_shortcut_key(shortcut)?;

        {
            let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
            let mut callbacks_guard = self.press_callbacks.lock().map_err(|e| e.to_string())?;

            if shortcuts_guard.contains_key(shortcut) {
                return Err(format!("Shortcut '{}' is already registered", shortcut));
            }

            shortcuts_guard.insert(shortcut.to_string(), key);
            callbacks_guard.insert(shortcut.to_string(), Arc::from(callback));
        }

        self.start_listener()?;
        Ok(())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
        let mut press_callbacks_guard = self.press_callbacks.lock().map_err(|e| e.to_string())?;
        let mut release_callbacks_guard = self.release_callbacks.lock().map_err(|e| e.to_string())?;

        shortcuts_guard.remove(shortcut);
        press_callbacks_guard.remove(shortcut);
        release_callbacks_guard.remove(shortcut);

        // Note: We don't stop the listener when all shortcuts are removed because
        // rdev::listen() blocks and doesn't provide a clean way to stop it.
        // The listener will continue running but won't call any callbacks.

        Ok(())
    }
}

impl ShortcutBackendExt for RdevShortcutBackend {
    fn register_with_release(
        &self,
        shortcut: &str,
        on_press: Box<dyn Fn() + Send + Sync>,
        on_release: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        let key = parse_shortcut_key(shortcut)?;

        {
            let mut shortcuts_guard = self.registered_shortcuts.lock().map_err(|e| e.to_string())?;
            let mut press_callbacks_guard = self.press_callbacks.lock().map_err(|e| e.to_string())?;
            let mut release_callbacks_guard = self.release_callbacks.lock().map_err(|e| e.to_string())?;

            if shortcuts_guard.contains_key(shortcut) {
                return Err(format!("Shortcut '{}' is already registered", shortcut));
            }

            shortcuts_guard.insert(shortcut.to_string(), key);
            press_callbacks_guard.insert(shortcut.to_string(), Arc::from(on_press));
            release_callbacks_guard.insert(shortcut.to_string(), Arc::from(on_release));
        }

        self.start_listener()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "rdev_backend_test.rs"]
mod tests;
