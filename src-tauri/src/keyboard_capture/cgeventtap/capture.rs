//! CGEventTap capture lifecycle management.
//!
//! Contains the CGEventTapCapture struct and its lifecycle methods.

use super::callback::{handle_cg_event, NX_SYSDEFINED};
use super::types::CapturedKeyEvent;
use super::{CONSUME_ESCAPE, ESCAPE_KEY_CODE};
use crate::keyboard_capture::permissions::{
    check_accessibility_permission, check_accessibility_permission_with_prompt,
    AccessibilityPermissionError,
};
use core_foundation::base::TCFType;
use core_foundation::mach_port::{CFMachPort, CFMachPortRef};
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopStop};
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy,
    CGEventType,
};
use foreign_types::ForeignType;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// CGEventMask type for raw FFI
type CGEventMask = u64;

/// Internal callback type for raw FFI
type CGEventTapCallBackInternal = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void;

/// Wrapper for boxed callback closure
type CGEventTapCallBackFn<'a> =
    Box<dyn Fn(CGEventTapProxy, CGEventType, &CGEvent) -> Option<CGEvent> + 'a>;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBackInternal,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

/// State shared between the capture thread and callback
pub struct CaptureState {
    /// Callback to invoke when a key event is captured
    pub callback: Option<Box<dyn Fn(CapturedKeyEvent) + Send + 'static>>,
    /// The run loop reference for stopping
    pub run_loop: Option<CFRunLoop>,
}

/// Handle to the CGEventTap keyboard capture system
pub struct CGEventTapCapture {
    /// Whether capture is currently active
    running: Arc<AtomicBool>,
    /// Shared state for the capture
    state: Arc<Mutex<CaptureState>>,
    /// Handle to the capture thread
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl CGEventTapCapture {
    /// Create a new CGEventTap capture instance
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(CaptureState {
                callback: None,
                run_loop: None,
            })),
            thread_handle: None,
        }
    }

    /// Start capturing keyboard events
    ///
    /// The callback will be invoked for each key event captured.
    /// Returns an error if capture is already running or if permission is not granted.
    pub fn start<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(CapturedKeyEvent) + Send + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return Err("CGEventTap capture is already running".to_string());
        }

        // Check Accessibility permission before starting.
        // In debug builds, skip the prompt to avoid popups during dev/test.
        // In release builds (production), prompt the user to grant permission.
        // Set HEYCAT_ACCESSIBILITY_PROMPT=1 to force the prompt in debug builds (for UX testing).
        let has_permission =
            if cfg!(debug_assertions) && std::env::var("HEYCAT_ACCESSIBILITY_PROMPT").is_err() {
                check_accessibility_permission()
            } else {
                check_accessibility_permission_with_prompt()
            };
        crate::info!("Accessibility permission check: {}", has_permission);
        if !has_permission {
            return Err(AccessibilityPermissionError::new().to_string());
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
            if let Err(e) = run_cgeventtap_loop(running, state) {
                crate::error!("CGEventTap capture error: {}", e);
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

        // Wait for thread to finish with timeout
        if let Some(handle) = self.thread_handle.take() {
            // Give the thread a reasonable time to finish
            let timeout = Duration::from_secs(2);
            let start = std::time::Instant::now();
            while !handle.is_finished() && start.elapsed() < timeout {
                std::thread::sleep(Duration::from_millis(10));
            }
            if handle.is_finished() {
                handle.join().map_err(|_| "Failed to join capture thread")?;
            }
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

impl Default for CGEventTapCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CGEventTapCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Internal callback for raw CGEventTap FFI
///
/// Returns:
/// - Event pointer: passes the event through to other applications
/// - null_mut(): blocks/consumes the event (requires DefaultTap mode)
unsafe extern "C" fn cg_event_tap_callback_internal(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event_ref: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void {
    let callback = user_info as *mut CGEventTapCallBackFn;
    let event = CGEvent::from_ptr(event_ref as *mut _);
    let new_event = (*callback)(_proxy, event_type, &event);

    match new_event {
        Some(returned_event) => {
            // Pass through the returned event to other applications
            ManuallyDrop::new(returned_event).as_ptr() as *mut c_void
        }
        None => {
            // Block/consume the event - prevent it from reaching other applications
            // Keep original event alive to avoid premature drop
            let _ = ManuallyDrop::new(event);
            std::ptr::null_mut()
        }
    }
}

/// Run the CGEventTap capture loop
fn run_cgeventtap_loop(
    running: Arc<AtomicBool>,
    state: Arc<Mutex<CaptureState>>,
) -> Result<(), String> {
    // Create event mask including KeyDown, KeyUp, FlagsChanged, and NSSystemDefined (for media keys)
    // CGEventType values: KeyDown=10, KeyUp=11, FlagsChanged=12, NSSystemDefined=14
    let event_mask: CGEventMask = (1 << CGEventType::KeyDown as u64)
        | (1 << CGEventType::KeyUp as u64)
        | (1 << CGEventType::FlagsChanged as u64)
        | (1 << NX_SYSDEFINED as u64); // NSSystemDefined for media keys

    // Clone state for the callback
    let callback_state = state.clone();

    // Create boxed callback closure
    // Returns Some(event) to pass through, None to block
    let callback: CGEventTapCallBackFn = Box::new(move |_proxy, event_type, event| {
        // Always process the event for hotkey detection first
        handle_cg_event(event_type, event, &callback_state);

        // Check if we should consume (block) Escape key events
        // This prevents Escape from reaching other apps during recording
        let event_type_raw = event_type as u32;
        if event_type_raw == 10 || event_type_raw == 11 {
            // KeyDown (10) or KeyUp (11)
            let key_code = event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            ) as u16;

            if key_code == ESCAPE_KEY_CODE && CONSUME_ESCAPE.load(Ordering::SeqCst) {
                crate::debug!("Blocking Escape key event (consume mode active)");
                return None; // Block the event
            }
        }

        // Pass through all other events
        Some(event.clone())
    });
    let cb = Box::new(callback);
    let cbr = Box::into_raw(cb);

    // Create the event tap with raw FFI (to support NSSystemDefined event type)
    let event_tap_ref = unsafe {
        CGEventTapCreate(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            event_mask,
            cg_event_tap_callback_internal,
            cbr as *mut c_void,
        )
    };

    if event_tap_ref.is_null() {
        // Clean up the callback box before returning error
        unsafe {
            let _ = Box::from_raw(cbr);
        }
        return Err(
            "Failed to create CGEventTap. Ensure Accessibility permission is granted.".to_string(),
        );
    }

    let mach_port = unsafe { CFMachPort::wrap_under_create_rule(event_tap_ref) };

    // Get the run loop source from the event tap
    let run_loop_source = mach_port
        .create_runloop_source(0)
        .map_err(|_| "Failed to create run loop source")?;

    // Get the current run loop and store it for later stopping
    let run_loop = CFRunLoop::get_current();
    if let Ok(mut guard) = state.lock() {
        guard.run_loop = Some(run_loop.clone());
    }

    // Register run loop globally for graceful shutdown on SIGINT
    crate::shutdown::register_cgeventtap_run_loop(run_loop.clone());

    // Add the source to the run loop (use DefaultMode which we'll run in)
    run_loop.add_source(&run_loop_source, unsafe { kCFRunLoopDefaultMode });

    // Enable the event tap
    unsafe {
        CGEventTapEnable(mach_port.as_concrete_TypeRef(), true);
    }

    crate::info!("CGEventTap keyboard capture started (with media key support)");

    // Run the loop until stopped
    // Use kCFRunLoopDefaultMode (not kCFRunLoopCommonModes which is for adding sources)
    while running.load(Ordering::SeqCst) {
        // Run for 1 second at a time, checking if we should stop
        CFRunLoop::run_in_mode(
            unsafe { kCFRunLoopDefaultMode },
            Duration::from_secs(1),
            false,
        );
    }

    // Cleanup: remove from run loop
    run_loop.remove_source(&run_loop_source, unsafe { kCFRunLoopDefaultMode });

    // Clean up the callback box
    unsafe {
        let _ = Box::from_raw(cbr);
    }

    crate::info!("CGEventTap keyboard capture stopped");
    Ok(())
}
