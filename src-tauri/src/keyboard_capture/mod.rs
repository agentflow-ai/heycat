// Keyboard capture module using IOKit HID for capturing fn key and other special keys
// This module provides low-level keyboard event capture that can detect keys that
// JavaScript's KeyboardEvent API cannot, such as the fn key on Mac keyboards.

use core_foundation::base::{kCFAllocatorDefault, CFRelease, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopStop};
use core_foundation::string::CFString;
use io_kit_sys::hid::base::IOHIDValueRef;
use io_kit_sys::hid::element::{IOHIDElementGetUsage, IOHIDElementGetUsagePage};
use io_kit_sys::hid::manager::{
    kIOHIDManagerOptionNone, IOHIDManagerClose, IOHIDManagerCreate,
    IOHIDManagerOpen, IOHIDManagerRegisterInputValueCallback,
    IOHIDManagerScheduleWithRunLoop, IOHIDManagerSetDeviceMatching,
    IOHIDManagerUnscheduleFromRunLoop,
};
use io_kit_sys::hid::usage_tables::{kHIDPage_GenericDesktop, kHIDUsage_GD_Keyboard};
use io_kit_sys::hid::value::{IOHIDValueGetElement, IOHIDValueGetIntegerValue};
use io_kit_sys::ret::kIOReturnSuccess;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Apple vendor-specific HID page for fn key
const APPLE_VENDOR_TOP_CASE_PAGE: u32 = 0xFF;
#[allow(dead_code)]
const APPLE_VENDOR_KEYBOARD_PAGE: u32 = 0xFF01;
const APPLE_FN_KEY_USAGE: u32 = 0x03;

// Standard HID keyboard page
const HID_KEYBOARD_PAGE: u32 = 0x07;

// Dictionary keys for matching (as string literals to avoid FFI pointer issues)
const IOHID_DEVICE_USAGE_PAGE_KEY: &str = "DeviceUsagePage";
const IOHID_DEVICE_USAGE_KEY: &str = "DeviceUsage";

/// Captured key event with all modifier information
#[derive(Debug, Clone, serde::Serialize)]
pub struct CapturedKeyEvent {
    /// The main key that was pressed (HID usage code)
    pub key_code: u32,
    /// Human-readable key name
    pub key_name: String,
    /// Whether fn key is pressed
    pub fn_key: bool,
    /// Whether command (meta) key is pressed
    pub command: bool,
    /// Whether control key is pressed
    pub control: bool,
    /// Whether alt/option key is pressed
    pub alt: bool,
    /// Whether shift key is pressed
    pub shift: bool,
    /// Whether this is a key press (true) or release (false)
    pub pressed: bool,
}

/// State shared between the capture thread and callback
struct CaptureState {
    /// Callback to invoke when a key event is captured
    callback: Option<Box<dyn Fn(CapturedKeyEvent) + Send + 'static>>,
    /// Current modifier state
    fn_pressed: bool,
    command_pressed: bool,
    control_pressed: bool,
    alt_pressed: bool,
    shift_pressed: bool,
    /// The run loop reference for stopping
    run_loop: Option<CFRunLoop>,
}

/// Handle to the keyboard capture system
pub struct KeyboardCapture {
    /// Whether capture is currently active
    running: Arc<AtomicBool>,
    /// Shared state for the capture
    state: Arc<Mutex<CaptureState>>,
    /// Handle to the capture thread
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl KeyboardCapture {
    /// Create a new keyboard capture instance
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(CaptureState {
                callback: None,
                fn_pressed: false,
                command_pressed: false,
                control_pressed: false,
                alt_pressed: false,
                shift_pressed: false,
                run_loop: None,
            })),
            thread_handle: None,
        }
    }

    /// Start capturing keyboard events
    ///
    /// The callback will be invoked for each key event captured.
    /// Returns an error if capture is already running or if IOHIDManager fails to initialize.
    pub fn start<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(CapturedKeyEvent) + Send + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return Err("Keyboard capture is already running".to_string());
        }

        // Store the callback
        {
            let mut state = self.state.lock().map_err(|e| e.to_string())?;
            state.callback = Some(Box::new(callback));
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let state = self.state.clone();

        let handle = thread::spawn(move || {
            if let Err(e) = run_capture_loop(running, state) {
                crate::error!("Keyboard capture error: {}", e);
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop capturing keyboard events
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(false, Ordering::SeqCst);

        // Stop the run loop
        if let Ok(state) = self.state.lock() {
            if let Some(ref run_loop) = state.run_loop {
                unsafe {
                    CFRunLoopStop(run_loop.as_concrete_TypeRef());
                }
            }
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|_| "Failed to join capture thread")?;
        }

        // Clear callback
        if let Ok(mut state) = self.state.lock() {
            state.callback = None;
            state.run_loop = None;
        }

        Ok(())
    }

    /// Check if capture is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for KeyboardCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Type alias for the HID input value callback
type IOHIDValueCallback = unsafe extern "C" fn(
    context: *mut c_void,
    result: i32,
    sender: *mut c_void,
    value: IOHIDValueRef,
);

/// Run the IOHIDManager capture loop
fn run_capture_loop(
    running: Arc<AtomicBool>,
    state: Arc<Mutex<CaptureState>>,
) -> Result<(), String> {
    unsafe {
        // Create the HID manager
        let manager = IOHIDManagerCreate(kCFAllocatorDefault, kIOHIDManagerOptionNone);
        if manager.is_null() {
            return Err("Failed to create IOHIDManager".to_string());
        }

        // Create matching dictionary for keyboards
        let usage_page = CFNumber::from(kHIDPage_GenericDesktop as i32);
        let usage = CFNumber::from(kHIDUsage_GD_Keyboard as i32);

        let keys = vec![
            CFString::new(IOHID_DEVICE_USAGE_PAGE_KEY),
            CFString::new(IOHID_DEVICE_USAGE_KEY),
        ];
        let values = vec![usage_page.as_CFType(), usage.as_CFType()];

        let matching_dict = CFDictionary::from_CFType_pairs(
            &keys
                .iter()
                .zip(values.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>(),
        );

        IOHIDManagerSetDeviceMatching(manager, matching_dict.as_concrete_TypeRef() as *const _);

        // Get the current run loop and store it for later stopping
        let run_loop = CFRunLoop::get_current();
        if let Ok(mut guard) = state.lock() {
            guard.run_loop = Some(run_loop.clone());
        }

        // Schedule with run loop
        IOHIDManagerScheduleWithRunLoop(
            manager,
            run_loop.as_concrete_TypeRef(),
            kCFRunLoopDefaultMode,
        );

        // Register callback - leak the Arc to pass as raw pointer
        let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;

        // Cast our callback to the expected function pointer type
        let callback_fn: IOHIDValueCallback = input_value_callback;
        IOHIDManagerRegisterInputValueCallback(manager, callback_fn, state_ptr);

        // Open the manager
        let result = IOHIDManagerOpen(manager, kIOHIDManagerOptionNone);
        if result != kIOReturnSuccess {
            // Clean up on failure
            let _ = Arc::from_raw(state_ptr as *const Mutex<CaptureState>);
            IOHIDManagerUnscheduleFromRunLoop(
                manager,
                run_loop.as_concrete_TypeRef(),
                kCFRunLoopDefaultMode,
            );
            CFRelease(manager as *const c_void);

            // kIOReturnNotPermitted = -536870203 (0xE00002C1)
            // This means the app doesn't have Input Monitoring permission
            const K_IO_RETURN_NOT_PERMITTED: i32 = -536870203;
            if result == K_IO_RETURN_NOT_PERMITTED {
                return Err(
                    "Input Monitoring permission required. Please grant permission in System Settings > Privacy & Security > Input Monitoring, then restart the app."
                        .to_string(),
                );
            }

            return Err(format!("Failed to open IOHIDManager: {}", result));
        }

        crate::info!("Keyboard capture started");

        // Run the loop until stopped
        while running.load(Ordering::SeqCst) {
            // Run the loop for a short interval to allow checking the running flag
            CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, Duration::from_millis(100), false);
        }

        // Clean up - create a null callback function pointer
        let null_callback: IOHIDValueCallback =
            std::mem::transmute::<*const (), IOHIDValueCallback>(std::ptr::null());
        IOHIDManagerRegisterInputValueCallback(manager, null_callback, std::ptr::null_mut());
        IOHIDManagerClose(manager, kIOHIDManagerOptionNone);
        IOHIDManagerUnscheduleFromRunLoop(
            manager,
            run_loop.as_concrete_TypeRef(),
            kCFRunLoopDefaultMode,
        );
        CFRelease(manager as *const c_void);

        // Recover the Arc we leaked
        let _ = Arc::from_raw(state_ptr as *const Mutex<CaptureState>);

        crate::info!("Keyboard capture stopped");
        Ok(())
    }
}

/// Callback invoked by IOHIDManager when an input value changes
unsafe extern "C" fn input_value_callback(
    context: *mut c_void,
    _result: i32,
    _sender: *mut c_void,
    value: IOHIDValueRef,
) {
    if context.is_null() || value.is_null() {
        return;
    }

    let state = &*(context as *const Mutex<CaptureState>);

    let element = IOHIDValueGetElement(value);
    if element.is_null() {
        return;
    }

    let usage_page = IOHIDElementGetUsagePage(element);
    let usage = IOHIDElementGetUsage(element);
    let int_value = IOHIDValueGetIntegerValue(value);
    let pressed = int_value != 0;

    // Process the key event
    if let Ok(mut guard) = state.lock() {
        // Check for fn key (Apple vendor-specific)
        if usage_page == APPLE_VENDOR_TOP_CASE_PAGE && usage == APPLE_FN_KEY_USAGE {
            guard.fn_pressed = pressed;

            // Emit fn key event
            if let Some(ref callback) = guard.callback {
                let event = CapturedKeyEvent {
                    key_code: usage,
                    key_name: "fn".to_string(),
                    fn_key: pressed,
                    command: guard.command_pressed,
                    control: guard.control_pressed,
                    alt: guard.alt_pressed,
                    shift: guard.shift_pressed,
                    pressed,
                };
                callback(event);
            }
            return;
        }

        // Check for standard keyboard keys
        if usage_page == HID_KEYBOARD_PAGE {
            // Update modifier state
            match usage {
                0xE0 => guard.control_pressed = pressed, // Left Control
                0xE1 => guard.shift_pressed = pressed,   // Left Shift
                0xE2 => guard.alt_pressed = pressed,     // Left Alt
                0xE3 => guard.command_pressed = pressed, // Left GUI (Command)
                0xE4 => guard.control_pressed = pressed, // Right Control
                0xE5 => guard.shift_pressed = pressed,   // Right Shift
                0xE6 => guard.alt_pressed = pressed,     // Right Alt
                0xE7 => guard.command_pressed = pressed, // Right GUI (Command)
                _ => {}
            }

            // Only emit events for non-modifier keys on press
            // (or for modifier keys if we want to track them)
            let key_name = hid_usage_to_key_name(usage);

            if let Some(ref callback) = guard.callback {
                let event = CapturedKeyEvent {
                    key_code: usage,
                    key_name,
                    fn_key: guard.fn_pressed,
                    command: guard.command_pressed,
                    control: guard.control_pressed,
                    alt: guard.alt_pressed,
                    shift: guard.shift_pressed,
                    pressed,
                };
                callback(event);
            }
        }
    }
}

/// Convert HID keyboard usage code to a human-readable key name
fn hid_usage_to_key_name(usage: u32) -> String {
    match usage {
        0x04..=0x1D => {
            // A-Z (0x04 = A, 0x1D = Z)
            let letter = (b'A' + (usage - 0x04) as u8) as char;
            letter.to_string()
        }
        0x1E..=0x27 => {
            // 1-9, 0 (0x1E = 1, 0x27 = 0)
            if usage == 0x27 {
                "0".to_string()
            } else {
                ((usage - 0x1E + 1) as u8).to_string()
            }
        }
        0x28 => "Enter".to_string(),
        0x29 => "Escape".to_string(),
        0x2A => "Backspace".to_string(),
        0x2B => "Tab".to_string(),
        0x2C => "Space".to_string(),
        0x2D => "-".to_string(),
        0x2E => "=".to_string(),
        0x2F => "[".to_string(),
        0x30 => "]".to_string(),
        0x31 => "\\".to_string(),
        0x33 => ";".to_string(),
        0x34 => "'".to_string(),
        0x35 => "`".to_string(),
        0x36 => ",".to_string(),
        0x37 => ".".to_string(),
        0x38 => "/".to_string(),
        0x39 => "CapsLock".to_string(),
        0x3A..=0x45 => format!("F{}", usage - 0x3A + 1), // F1-F12
        0x46 => "PrintScreen".to_string(),
        0x47 => "ScrollLock".to_string(),
        0x48 => "Pause".to_string(),
        0x49 => "Insert".to_string(),
        0x4A => "Home".to_string(),
        0x4B => "PageUp".to_string(),
        0x4C => "Delete".to_string(),
        0x4D => "End".to_string(),
        0x4E => "PageDown".to_string(),
        0x4F => "Right".to_string(),
        0x50 => "Left".to_string(),
        0x51 => "Down".to_string(),
        0x52 => "Up".to_string(),
        0xE0 => "Control".to_string(),
        0xE1 => "Shift".to_string(),
        0xE2 => "Alt".to_string(),
        0xE3 => "Command".to_string(),
        0xE4 => "Control".to_string(),
        0xE5 => "Shift".to_string(),
        0xE6 => "Alt".to_string(),
        0xE7 => "Command".to_string(),
        _ => format!("Key(0x{:02X})", usage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_usage_to_key_name() {
        assert_eq!(hid_usage_to_key_name(0x04), "A");
        assert_eq!(hid_usage_to_key_name(0x1D), "Z");
        assert_eq!(hid_usage_to_key_name(0x1E), "1");
        assert_eq!(hid_usage_to_key_name(0x27), "0");
        assert_eq!(hid_usage_to_key_name(0x28), "Enter");
        assert_eq!(hid_usage_to_key_name(0x29), "Escape");
        assert_eq!(hid_usage_to_key_name(0x3A), "F1");
        assert_eq!(hid_usage_to_key_name(0x45), "F12");
        assert_eq!(hid_usage_to_key_name(0xE3), "Command");
    }

    #[test]
    fn test_captured_key_event_serialization() {
        let event = CapturedKeyEvent {
            key_code: 0x04,
            key_name: "A".to_string(),
            fn_key: true,
            command: false,
            control: false,
            alt: false,
            shift: false,
            pressed: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"fn_key\":true"));
        assert!(json.contains("\"key_name\":\"A\""));
    }
}
