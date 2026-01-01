//! CGEventTap callback handling.
//!
//! Contains the event callback logic for processing keyboard events.

#[allow(deprecated)]
use cocoa::appkit::NSEvent;
#[allow(deprecated)]
use cocoa::base::nil;
use core_graphics::event::CGEvent;
use core_graphics::event::CGEventType;
use foreign_types::ForeignType;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use super::capture::CaptureState;
use super::keycodes::{keycode_to_name, media_keycode_to_name};
use super::modifiers::{determine_modifier_key_state, ModifierFlags};
use super::types::CapturedKeyEvent;

// NSSystemDefined event constants (from IOKit/hidsystem)
pub const NX_SYSDEFINED: u32 = 14; // NSSystemDefined event type
pub const NX_SUBTYPE_AUX_CONTROL_BUTTONS: i16 = 8; // Media key subtype

/// Handle a CGEvent and convert it to CapturedKeyEvent
pub fn handle_cg_event(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<Mutex<CaptureState>>,
) {
    // Track timing to diagnose keyboard freezing issues
    let start = std::time::Instant::now();

    // Wrap everything in catch_unwind to prevent crashes from taking down the app
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handle_cg_event_inner(event_type, event, state);
    }));

    if let Err(e) = result {
        crate::error!("CGEventTap callback panicked: {:?}", e);
    }

    // Warn if callback took too long (could cause keyboard freeze)
    let elapsed = start.elapsed();
    if elapsed.as_millis() > 10 {
        crate::warn!(
            "handle_cg_event took {:?} - SLOW! This may cause keyboard freeze",
            elapsed
        );
    }
}

/// Inner implementation of handle_cg_event
pub fn handle_cg_event_inner(
    event_type: CGEventType,
    event: &CGEvent,
    state: &Arc<Mutex<CaptureState>>,
) {
    let flags = event.get_flags();
    let flags_raw = flags.bits();

    // Extract modifier state from flags
    let mods = ModifierFlags::from_cg_flags(flags_raw);

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
                fn_key: mods.fn_key,
                command: mods.command,
                command_left: mods.command_left,
                command_right: mods.command_right,
                control: mods.control,
                control_left: mods.control_left,
                control_right: mods.control_right,
                alt: mods.alt,
                alt_left: mods.alt_left,
                alt_right: mods.alt_right,
                shift: mods.shift,
                shift_left: mods.shift_left,
                shift_right: mods.shift_right,
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
                fn_key: mods.fn_key,
                command: mods.command,
                command_left: mods.command_left,
                command_right: mods.command_right,
                control: mods.control,
                control_left: mods.control_left,
                control_right: mods.control_right,
                alt: mods.alt,
                alt_left: mods.alt_left,
                alt_right: mods.alt_right,
                shift: mods.shift,
                shift_left: mods.shift_left,
                shift_right: mods.shift_right,
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
                    fn_key: mods.fn_key,
                    command: mods.command,
                    command_left: mods.command_left,
                    command_right: mods.command_right,
                    control: mods.control,
                    control_left: mods.control_left,
                    control_right: mods.control_right,
                    alt: mods.alt,
                    alt_left: mods.alt_left,
                    alt_right: mods.alt_right,
                    shift: mods.shift,
                    shift_left: mods.shift_left,
                    shift_right: mods.shift_right,
                    pressed,
                    is_media_key: true,
                }
            }
        }
        _ => return,
    };

    // Invoke callback with the captured event
    // IMPORTANT: Use try_lock() instead of lock() to avoid blocking the CGEventTap callback.
    // If we block here, ALL keyboard input system-wide will freeze until we return.
    // If the lock is contended, we skip this event rather than freezing the keyboard.
    if let Ok(guard) = state.try_lock() {
        if let Some(ref callback) = guard.callback {
            callback(captured_event);
        } else {
            crate::warn!("CGEventTap callback is None!");
        }
    } else {
        crate::trace!("Skipping key event - state lock contended");
    }
}
