// CGEventTap-based keyboard event capture for macOS
//
// This module uses CGEventTap to capture ALL keyboard events including:
// - Regular keys (letters, numbers, symbols, special keys)
// - Modifier keys with left/right distinction
// - fn/Globe key via FlagsChanged
// - Media keys (volume, brightness, play/pause) via NSSystemDefined
// - Full modifier state tracking
//
// CGEventTap requires Accessibility permission (System Settings > Privacy & Security > Accessibility)

use super::permissions::{
    check_accessibility_permission, check_accessibility_permission_with_prompt,
    AccessibilityPermissionError,
};
#[allow(deprecated)]
use cocoa::appkit::NSEvent;
#[allow(deprecated)]
use cocoa::base::nil;
use core_foundation::base::TCFType;
use core_foundation::mach_port::{CFMachPort, CFMachPortRef};
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopStop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};
use foreign_types::ForeignType;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Standard modifier flags from CGEvent
const CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x00020000;
const CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x00040000;
const CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;
const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;
const CG_EVENT_FLAG_MASK_SECONDARY_FN: u64 = 0x00800000;

// Left/Right device flags (from IOKit NX_DEVICE*KEYMASK constants)
const NX_DEVICELSHIFTKEYMASK: u64 = 0x00000002;
const NX_DEVICERSHIFTKEYMASK: u64 = 0x00000004;
const NX_DEVICELCTLKEYMASK: u64 = 0x00000001;
const NX_DEVICERCTLKEYMASK: u64 = 0x00002000;
const NX_DEVICELALTKEYMASK: u64 = 0x00000020;
const NX_DEVICERALTKEYMASK: u64 = 0x00000040;
const NX_DEVICELCMDKEYMASK: u64 = 0x00000008;
const NX_DEVICERCMDKEYMASK: u64 = 0x00000010;

// NSSystemDefined event constants (from IOKit/hidsystem)
const NX_SYSDEFINED: u32 = 14; // NSSystemDefined event type
const NX_SUBTYPE_AUX_CONTROL_BUTTONS: i16 = 8; // Media key subtype

// Media key codes (from IOKit/hidsystem/ev_keymap.h NX_KEYTYPE_*)
const NX_KEYTYPE_SOUND_UP: u32 = 0;
const NX_KEYTYPE_SOUND_DOWN: u32 = 1;
const NX_KEYTYPE_BRIGHTNESS_UP: u32 = 2;
const NX_KEYTYPE_BRIGHTNESS_DOWN: u32 = 3;
const NX_KEYTYPE_MUTE: u32 = 7;
const NX_KEYTYPE_PLAY: u32 = 16;
const NX_KEYTYPE_NEXT: u32 = 17;
const NX_KEYTYPE_PREVIOUS: u32 = 18;
const NX_KEYTYPE_FAST: u32 = 19;
const NX_KEYTYPE_REWIND: u32 = 20;
const NX_KEYTYPE_ILLUMINATION_UP: u32 = 21;
const NX_KEYTYPE_ILLUMINATION_DOWN: u32 = 22;

/// CGEventMask type for raw FFI
type CGEventMask = u64;

/// Escape key code on macOS
const ESCAPE_KEY_CODE: u16 = 53;

/// Global flag to control Escape key consumption during recording.
/// When true, Escape key events are blocked from reaching other applications.
/// This flag is thread-safe and can be set from the HotkeyIntegration layer.
static CONSUME_ESCAPE: AtomicBool = AtomicBool::new(false);

/// Set whether Escape key events should be consumed (blocked from other apps).
/// Call with `true` when recording starts, `false` when recording stops/cancels.
pub fn set_consume_escape(consume: bool) {
    CONSUME_ESCAPE.store(consume, Ordering::SeqCst);
    crate::debug!("Escape key consume mode: {}", consume);
}

/// Get the current state of the Escape key consumption flag.
/// Returns true if Escape events are being blocked.
#[cfg(test)]
pub fn get_consume_escape() -> bool {
    CONSUME_ESCAPE.load(Ordering::SeqCst)
}

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

/// Captured key event with full modifier information including left/right distinction
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CapturedKeyEvent {
    /// The key code (CGKeyCode)
    pub key_code: u32,
    /// Human-readable key name
    pub key_name: String,
    /// Whether fn key is pressed
    pub fn_key: bool,
    /// Whether any command key is pressed
    pub command: bool,
    /// Whether left command is pressed
    pub command_left: bool,
    /// Whether right command is pressed
    pub command_right: bool,
    /// Whether any control key is pressed
    pub control: bool,
    /// Whether left control is pressed
    pub control_left: bool,
    /// Whether right control is pressed
    pub control_right: bool,
    /// Whether any alt/option key is pressed
    pub alt: bool,
    /// Whether left alt/option is pressed
    pub alt_left: bool,
    /// Whether right alt/option is pressed
    pub alt_right: bool,
    /// Whether any shift key is pressed
    pub shift: bool,
    /// Whether left shift is pressed
    pub shift_left: bool,
    /// Whether right shift is pressed
    pub shift_right: bool,
    /// Whether this is a key press (true) or release (false)
    pub pressed: bool,
    /// Whether this is a media key (volume, brightness, etc.)
    pub is_media_key: bool,
}

/// State shared between the capture thread and callback
struct CaptureState {
    /// Callback to invoke when a key event is captured
    callback: Option<Box<dyn Fn(CapturedKeyEvent) + Send + 'static>>,
    /// The run loop reference for stopping
    run_loop: Option<CFRunLoop>,
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

    // Debug logging disabled
    // crate::info!(
    //     "CGEventTap creating with event_mask=0x{:x}, KeyDown={}, KeyUp={}, FlagsChanged={}",
    //     event_mask,
    //     CGEventType::KeyDown as u64,
    //     CGEventType::KeyUp as u64,
    //     CGEventType::FlagsChanged as u64
    // );

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

/// Handle a CGEvent and convert it to CapturedKeyEvent
fn handle_cg_event(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<Mutex<CaptureState>>,
) {
    // Wrap everything in catch_unwind to prevent crashes from taking down the app
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handle_cg_event_inner(event_type, event, state);
    }));

    if let Err(e) = result {
        crate::error!("CGEventTap callback panicked: {:?}", e);
    }
}

/// Inner implementation of handle_cg_event
fn handle_cg_event_inner(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<Mutex<CaptureState>>,
) {
    // Debug logging disabled for performance
    // crate::info!("CGEventTap callback invoked: event_type={:?}", event_type);

    let flags = event.get_flags();
    let flags_raw = flags.bits();

    // Extract modifier state from flags
    let fn_key = (flags_raw & CG_EVENT_FLAG_MASK_SECONDARY_FN) != 0;
    let command = (flags_raw & CG_EVENT_FLAG_MASK_COMMAND) != 0;
    let control = (flags_raw & CG_EVENT_FLAG_MASK_CONTROL) != 0;
    let alt = (flags_raw & CG_EVENT_FLAG_MASK_ALTERNATE) != 0;
    let shift = (flags_raw & CG_EVENT_FLAG_MASK_SHIFT) != 0;

    // Extract left/right distinction from device flags
    let command_left = (flags_raw & NX_DEVICELCMDKEYMASK) != 0;
    let command_right = (flags_raw & NX_DEVICERCMDKEYMASK) != 0;
    let control_left = (flags_raw & NX_DEVICELCTLKEYMASK) != 0;
    let control_right = (flags_raw & NX_DEVICERCTLKEYMASK) != 0;
    let alt_left = (flags_raw & NX_DEVICELALTKEYMASK) != 0;
    let alt_right = (flags_raw & NX_DEVICERALTKEYMASK) != 0;
    let shift_left = (flags_raw & NX_DEVICELSHIFTKEYMASK) != 0;
    let shift_right = (flags_raw & NX_DEVICERSHIFTKEYMASK) != 0;

    // Get raw event type value for comparison (CGEventType doesn't implement PartialEq)
    let event_type_raw = event_type as u32;

    let captured_event = match event_type_raw {
        10 | 11 => {
            // KeyDown (10) or KeyUp (11)
            let key_code = event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            ) as u32;

            let pressed = event_type_raw == 10; // KeyDown
            let key_name = keycode_to_name(key_code);

            CapturedKeyEvent {
                key_code,
                key_name,
                fn_key,
                command,
                command_left,
                command_right,
                control,
                control_left,
                control_right,
                alt,
                alt_left,
                alt_right,
                shift,
                shift_left,
                shift_right,
                pressed,
                is_media_key: false,
            }
        }
        12 => {
            // FlagsChanged - we need to determine which modifier key changed
            let key_code = event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            ) as u32;

            let (key_name, pressed) = determine_modifier_key_state(key_code, flags_raw);

            CapturedKeyEvent {
                key_code,
                key_name,
                fn_key,
                command,
                command_left,
                command_right,
                control,
                control_left,
                control_right,
                alt,
                alt_left,
                alt_right,
                shift,
                shift_left,
                shift_right,
                pressed,
                is_media_key: false,
            }
        }
        14 => {
            // NSSystemDefined - handle media keys via NSEvent
            // We need to convert CGEvent to NSEvent to extract data1 and subtype
            #[allow(deprecated)]
            let cg_event_ptr = event.as_ptr() as *mut c_void;

            #[allow(deprecated)]
            unsafe {
                let ns_event: cocoa::base::id = NSEvent::eventWithCGEvent_(nil, cg_event_ptr);
                if ns_event == nil {
                    return;
                }

                let subtype = NSEvent::subtype(ns_event) as i16;
                if subtype != NX_SUBTYPE_AUX_CONTROL_BUTTONS {
                    return; // Not a media key event
                }

                let data1 = NSEvent::data1(ns_event);

                // Extract key code and state from data1
                // data1 format: upper 16 bits = key code, lower 16 bits = flags
                let key_code = ((data1 as u64 & 0xFFFF0000) >> 16) as u32;
                let key_flags = (data1 as u64 & 0x0000FFFF) as u32;

                // Key state: ((flags & 0xFF00) >> 8) == 0xA means pressed, 0xB means released
                let key_state = (key_flags & 0xFF00) >> 8;
                let pressed = key_state == 0x0A;

                let key_name = media_keycode_to_name(key_code);

                CapturedKeyEvent {
                    key_code,
                    key_name,
                    fn_key,
                    command,
                    command_left,
                    command_right,
                    control,
                    control_left,
                    control_right,
                    alt,
                    alt_left,
                    alt_right,
                    shift,
                    shift_left,
                    shift_right,
                    pressed,
                    is_media_key: true,
                }
            }
        }
        _ => return,
    };

    // Invoke callback with the captured event
    // Debug logging disabled for performance
    // crate::info!(
    //     "CGEventTap emitting event: key_name={}, pressed={}",
    //     captured_event.key_name,
    //     captured_event.pressed
    // );
    if let Ok(guard) = state.lock() {
        if let Some(ref callback) = guard.callback {
            callback(captured_event);
        } else {
            crate::warn!("CGEventTap callback is None!");
        }
    } else {
        crate::warn!("CGEventTap failed to lock state!");
    }
}

/// Determine which modifier key changed and whether it was pressed or released
fn determine_modifier_key_state(key_code: u32, flags: u64) -> (String, bool) {
    match key_code {
        // Shift keys
        56 => (
            "Shift".to_string(),
            (flags & NX_DEVICELSHIFTKEYMASK) != 0,
        ), // Left Shift
        60 => (
            "Shift".to_string(),
            (flags & NX_DEVICERSHIFTKEYMASK) != 0,
        ), // Right Shift
        // Control keys
        59 => (
            "Control".to_string(),
            (flags & NX_DEVICELCTLKEYMASK) != 0,
        ), // Left Control
        62 => (
            "Control".to_string(),
            (flags & NX_DEVICERCTLKEYMASK) != 0,
        ), // Right Control
        // Alt/Option keys
        58 => (
            "Alt".to_string(),
            (flags & NX_DEVICELALTKEYMASK) != 0,
        ), // Left Alt
        61 => (
            "Alt".to_string(),
            (flags & NX_DEVICERALTKEYMASK) != 0,
        ), // Right Alt
        // Command keys
        55 => (
            "Command".to_string(),
            (flags & NX_DEVICELCMDKEYMASK) != 0,
        ), // Left Command
        54 => (
            "Command".to_string(),
            (flags & NX_DEVICERCMDKEYMASK) != 0,
        ), // Right Command
        // Caps Lock
        57 => (
            "CapsLock".to_string(),
            (flags & CGEventFlags::CGEventFlagAlphaShift.bits()) != 0,
        ),
        // fn key - detected via the secondary fn flag
        // Key code 63 is traditional fn, 179 is Globe key on newer Macs
        63 | 179 => (
            "fn".to_string(),
            (flags & CG_EVENT_FLAG_MASK_SECONDARY_FN) != 0,
        ),
        _ => (format!("Modifier({})", key_code), true),
    }
}

/// Convert media key code to human-readable name
/// Media key codes are from IOKit/hidsystem/ev_keymap.h (NX_KEYTYPE_*)
fn media_keycode_to_name(key_code: u32) -> String {
    match key_code {
        NX_KEYTYPE_SOUND_UP => "VolumeUp".to_string(),
        NX_KEYTYPE_SOUND_DOWN => "VolumeDown".to_string(),
        NX_KEYTYPE_MUTE => "Mute".to_string(),
        NX_KEYTYPE_BRIGHTNESS_UP => "BrightnessUp".to_string(),
        NX_KEYTYPE_BRIGHTNESS_DOWN => "BrightnessDown".to_string(),
        NX_KEYTYPE_PLAY => "PlayPause".to_string(),
        NX_KEYTYPE_NEXT => "NextTrack".to_string(),
        NX_KEYTYPE_PREVIOUS => "PreviousTrack".to_string(),
        NX_KEYTYPE_FAST => "FastForward".to_string(),
        NX_KEYTYPE_REWIND => "Rewind".to_string(),
        NX_KEYTYPE_ILLUMINATION_UP => "KeyboardBrightnessUp".to_string(),
        NX_KEYTYPE_ILLUMINATION_DOWN => "KeyboardBrightnessDown".to_string(),
        _ => format!("MediaKey({})", key_code),
    }
}

/// Convert macOS key code to human-readable key name
pub fn keycode_to_name(key_code: u32) -> String {
    match key_code {
        // Letters (A-Z)
        0 => "A".to_string(),
        1 => "S".to_string(),
        2 => "D".to_string(),
        3 => "F".to_string(),
        4 => "H".to_string(),
        5 => "G".to_string(),
        6 => "Z".to_string(),
        7 => "X".to_string(),
        8 => "C".to_string(),
        9 => "V".to_string(),
        11 => "B".to_string(),
        12 => "Q".to_string(),
        13 => "W".to_string(),
        14 => "E".to_string(),
        15 => "R".to_string(),
        16 => "Y".to_string(),
        17 => "T".to_string(),
        31 => "O".to_string(),
        32 => "U".to_string(),
        34 => "I".to_string(),
        35 => "P".to_string(),
        37 => "L".to_string(),
        38 => "J".to_string(),
        40 => "K".to_string(),
        45 => "N".to_string(),
        46 => "M".to_string(),

        // Numbers (top row)
        18 => "1".to_string(),
        19 => "2".to_string(),
        20 => "3".to_string(),
        21 => "4".to_string(),
        22 => "6".to_string(),
        23 => "5".to_string(),
        24 => "=".to_string(),
        25 => "9".to_string(),
        26 => "7".to_string(),
        27 => "-".to_string(),
        28 => "8".to_string(),
        29 => "0".to_string(),

        // Punctuation and symbols
        30 => "]".to_string(),
        33 => "[".to_string(),
        39 => "'".to_string(),
        41 => ";".to_string(),
        42 => "\\".to_string(),
        43 => ",".to_string(),
        44 => "/".to_string(),
        47 => ".".to_string(),
        50 => "`".to_string(),

        // Special keys
        36 => "Enter".to_string(),
        48 => "Tab".to_string(),
        49 => "Space".to_string(),
        51 => "Backspace".to_string(),
        53 => "Escape".to_string(),

        // Modifier keys
        54 => "Command".to_string(),  // Right Command
        55 => "Command".to_string(),  // Left Command
        56 => "Shift".to_string(),    // Left Shift
        57 => "CapsLock".to_string(),
        58 => "Alt".to_string(),      // Left Alt/Option
        59 => "Control".to_string(),  // Left Control
        60 => "Shift".to_string(),    // Right Shift
        61 => "Alt".to_string(),      // Right Alt/Option
        62 => "Control".to_string(),  // Right Control
        63 | 179 => "fn".to_string(),       // fn/Globe key (179 on newer Macs)

        // Function keys
        122 => "F1".to_string(),
        120 => "F2".to_string(),
        99 => "F3".to_string(),
        118 => "F4".to_string(),
        96 => "F5".to_string(),
        97 => "F6".to_string(),
        98 => "F7".to_string(),
        100 => "F8".to_string(),
        101 => "F9".to_string(),
        109 => "F10".to_string(),
        103 => "F11".to_string(),
        111 => "F12".to_string(),
        105 => "F13".to_string(),
        107 => "F14".to_string(),
        113 => "F15".to_string(),
        106 => "F16".to_string(),
        64 => "F17".to_string(),
        79 => "F18".to_string(),
        80 => "F19".to_string(),

        // Navigation keys
        123 => "Left".to_string(),
        124 => "Right".to_string(),
        125 => "Down".to_string(),
        126 => "Up".to_string(),
        115 => "Home".to_string(),
        116 => "PageUp".to_string(),
        117 => "Delete".to_string(),   // Forward Delete
        119 => "End".to_string(),
        121 => "PageDown".to_string(),

        // Numpad keys
        65 => "Numpad.".to_string(),
        67 => "Numpad*".to_string(),
        69 => "Numpad+".to_string(),
        71 => "NumpadClear".to_string(),
        75 => "Numpad/".to_string(),
        76 => "NumpadEnter".to_string(),
        78 => "Numpad-".to_string(),
        81 => "Numpad=".to_string(),
        82 => "Numpad0".to_string(),
        83 => "Numpad1".to_string(),
        84 => "Numpad2".to_string(),
        85 => "Numpad3".to_string(),
        86 => "Numpad4".to_string(),
        87 => "Numpad5".to_string(),
        88 => "Numpad6".to_string(),
        89 => "Numpad7".to_string(),
        91 => "Numpad8".to_string(),
        92 => "Numpad9".to_string(),

        // Other
        10 => "Section".to_string(),  // § key (ISO keyboards)
        52 => "International".to_string(),  // International key
        102 => "Help".to_string(),  // Help key (older keyboards)
        110 => "ContextMenu".to_string(),

        _ => format!("Key({})", key_code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_to_name_letters() {
        assert_eq!(keycode_to_name(0), "A");
        assert_eq!(keycode_to_name(1), "S");
        assert_eq!(keycode_to_name(6), "Z");
        assert_eq!(keycode_to_name(12), "Q");
    }

    #[test]
    fn test_keycode_to_name_numbers() {
        assert_eq!(keycode_to_name(18), "1");
        assert_eq!(keycode_to_name(29), "0");
        assert_eq!(keycode_to_name(23), "5");
    }

    #[test]
    fn test_keycode_to_name_function_keys() {
        assert_eq!(keycode_to_name(122), "F1");
        assert_eq!(keycode_to_name(96), "F5");
        assert_eq!(keycode_to_name(111), "F12");
        assert_eq!(keycode_to_name(80), "F19");
    }

    #[test]
    fn test_keycode_to_name_special_keys() {
        assert_eq!(keycode_to_name(36), "Enter");
        assert_eq!(keycode_to_name(49), "Space");
        assert_eq!(keycode_to_name(48), "Tab");
        assert_eq!(keycode_to_name(53), "Escape");
        assert_eq!(keycode_to_name(51), "Backspace");
    }

    #[test]
    fn test_keycode_to_name_modifiers() {
        assert_eq!(keycode_to_name(55), "Command");  // Left
        assert_eq!(keycode_to_name(54), "Command");  // Right
        assert_eq!(keycode_to_name(56), "Shift");    // Left
        assert_eq!(keycode_to_name(60), "Shift");    // Right
        assert_eq!(keycode_to_name(58), "Alt");      // Left
        assert_eq!(keycode_to_name(61), "Alt");      // Right
        assert_eq!(keycode_to_name(59), "Control");  // Left
        assert_eq!(keycode_to_name(62), "Control");  // Right
        assert_eq!(keycode_to_name(63), "fn");
    }

    #[test]
    fn test_keycode_to_name_navigation() {
        assert_eq!(keycode_to_name(123), "Left");
        assert_eq!(keycode_to_name(124), "Right");
        assert_eq!(keycode_to_name(125), "Down");
        assert_eq!(keycode_to_name(126), "Up");
    }

    #[test]
    fn test_keycode_to_name_numpad() {
        assert_eq!(keycode_to_name(82), "Numpad0");
        assert_eq!(keycode_to_name(83), "Numpad1");
        assert_eq!(keycode_to_name(92), "Numpad9");
        assert_eq!(keycode_to_name(76), "NumpadEnter");
    }

    #[test]
    fn test_keycode_to_name_unknown() {
        assert_eq!(keycode_to_name(255), "Key(255)");
    }

    #[test]
    fn test_captured_key_event_default() {
        let event = CapturedKeyEvent::default();
        assert_eq!(event.key_code, 0);
        assert_eq!(event.key_name, "");
        assert!(!event.fn_key);
        assert!(!event.command);
        assert!(!event.command_left);
        assert!(!event.command_right);
        assert!(!event.pressed);
        assert!(!event.is_media_key);
    }

    #[test]
    fn test_captured_key_event_serialization() {
        let event = CapturedKeyEvent {
            key_code: 0,
            key_name: "A".to_string(),
            fn_key: false,
            command: true,
            command_left: true,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: true,
            shift_left: false,
            shift_right: true,
            pressed: true,
            is_media_key: false,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"key_name\":\"A\""));
        assert!(json.contains("\"command\":true"));
        assert!(json.contains("\"command_left\":true"));
        assert!(json.contains("\"command_right\":false"));
        assert!(json.contains("\"shift_right\":true"));
    }

    #[test]
    fn test_determine_modifier_key_state_left_shift() {
        let (name, pressed) = determine_modifier_key_state(56, NX_DEVICELSHIFTKEYMASK);
        assert_eq!(name, "Shift");
        assert!(pressed);

        let (_, released) = determine_modifier_key_state(56, 0);
        assert!(!released);
    }

    #[test]
    fn test_determine_modifier_key_state_right_shift() {
        let (name, pressed) = determine_modifier_key_state(60, NX_DEVICERSHIFTKEYMASK);
        assert_eq!(name, "Shift");
        assert!(pressed);
    }

    #[test]
    fn test_determine_modifier_key_state_left_command() {
        let (name, pressed) = determine_modifier_key_state(55, NX_DEVICELCMDKEYMASK);
        assert_eq!(name, "Command");
        assert!(pressed);
    }

    #[test]
    fn test_determine_modifier_key_state_right_command() {
        let (name, pressed) = determine_modifier_key_state(54, NX_DEVICERCMDKEYMASK);
        assert_eq!(name, "Command");
        assert!(pressed);
    }

    #[test]
    fn test_determine_modifier_key_state_fn() {
        let (name, pressed) = determine_modifier_key_state(63, CG_EVENT_FLAG_MASK_SECONDARY_FN);
        assert_eq!(name, "fn");
        assert!(pressed);
    }

    #[test]
    fn test_cgeventtap_capture_new_not_running() {
        let capture = CGEventTapCapture::new();
        assert!(!capture.is_running());
    }

    #[test]
    fn test_cgeventtap_capture_stop_when_not_running() {
        let mut capture = CGEventTapCapture::new();
        // Stopping when not running should be a no-op
        assert!(capture.stop().is_ok());
    }

    #[test]
    fn test_media_keycode_to_name_volume() {
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_SOUND_UP), "VolumeUp");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_SOUND_DOWN), "VolumeDown");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_MUTE), "Mute");
    }

    #[test]
    fn test_media_keycode_to_name_brightness() {
        assert_eq!(
            media_keycode_to_name(NX_KEYTYPE_BRIGHTNESS_UP),
            "BrightnessUp"
        );
        assert_eq!(
            media_keycode_to_name(NX_KEYTYPE_BRIGHTNESS_DOWN),
            "BrightnessDown"
        );
    }

    #[test]
    fn test_media_keycode_to_name_playback() {
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_PLAY), "PlayPause");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_NEXT), "NextTrack");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_PREVIOUS), "PreviousTrack");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_FAST), "FastForward");
        assert_eq!(media_keycode_to_name(NX_KEYTYPE_REWIND), "Rewind");
    }

    #[test]
    fn test_media_keycode_to_name_keyboard_backlight() {
        assert_eq!(
            media_keycode_to_name(NX_KEYTYPE_ILLUMINATION_UP),
            "KeyboardBrightnessUp"
        );
        assert_eq!(
            media_keycode_to_name(NX_KEYTYPE_ILLUMINATION_DOWN),
            "KeyboardBrightnessDown"
        );
    }

    #[test]
    fn test_media_keycode_to_name_unknown() {
        assert_eq!(media_keycode_to_name(255), "MediaKey(255)");
    }

    #[test]
    fn test_captured_key_event_media_key() {
        let event = CapturedKeyEvent {
            key_code: NX_KEYTYPE_SOUND_UP,
            key_name: "VolumeUp".to_string(),
            fn_key: false,
            command: false,
            command_left: false,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: false,
            shift_left: false,
            shift_right: false,
            pressed: true,
            is_media_key: true,
        };

        assert_eq!(event.key_code, 0);
        assert_eq!(event.key_name, "VolumeUp");
        assert!(event.pressed);
        assert!(event.is_media_key);
    }

    // === Escape key consumption tests ===

    #[test]
    fn test_consume_escape_default_false() {
        // Reset to known state first
        set_consume_escape(false);
        // Default state should be false (Escape passes through)
        assert!(!get_consume_escape());
    }

    #[test]
    fn test_consume_escape_set_true() {
        // Set consume mode to true
        set_consume_escape(true);
        assert!(get_consume_escape());
        // Clean up
        set_consume_escape(false);
    }

    #[test]
    fn test_consume_escape_set_false() {
        // First set to true
        set_consume_escape(true);
        assert!(get_consume_escape());
        // Then set back to false
        set_consume_escape(false);
        assert!(!get_consume_escape());
    }

    #[test]
    fn test_escape_key_code_constant() {
        // Verify the ESCAPE_KEY_CODE constant is correct (53 on macOS)
        assert_eq!(ESCAPE_KEY_CODE, 53);
    }

    #[test]
    fn test_keycode_to_name_escape() {
        // Verify Escape key is properly mapped
        assert_eq!(keycode_to_name(ESCAPE_KEY_CODE as u32), "Escape");
    }
}
