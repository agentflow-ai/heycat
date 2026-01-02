//! Swift FFI bindings module.
//!
//! This module provides safe Rust wrappers around Swift functions
//! compiled via swift-rs.

use swift_rs::SRString;

// Define the FFI function signatures
// SRString is the Swift-Rs string type that can be safely passed across FFI

// Hello function for testing
swift_rs::swift!(fn swift_hello() -> SRString);

// =============================================================================
// System Wake Notification
// =============================================================================

/// Type alias for the wake callback function pointer.
/// The callback takes no arguments and returns void.
/// Used by: create-wake-handler-module-for-sleep-wake-events (spec #3)
#[allow(dead_code)]
pub type WakeCallback = extern "C" fn();

// Pass the callback as a raw pointer since swift_rs doesn't support fn pointers directly
swift_rs::swift!(fn swift_register_wake_callback(callback: *const std::ffi::c_void));
swift_rs::swift!(fn swift_unregister_wake_callback());

// =============================================================================
// Audio Device Change Notification
// =============================================================================

/// Type alias for the device change callback function pointer.
/// The callback takes no arguments and returns void.
/// Used by: restart-audio-engine-on-device-change spec
#[allow(dead_code)]
pub type DeviceChangeCallback = extern "C" fn();

// Pass the callback as a raw pointer since swift_rs doesn't support fn pointers directly
swift_rs::swift!(fn swift_register_device_change_callback(callback: *const std::ffi::c_void));
swift_rs::swift!(fn swift_unregister_device_change_callback());

// Audio device enumeration functions
swift_rs::swift!(fn swift_refresh_audio_devices() -> i64);
swift_rs::swift!(fn swift_get_device_name(index: i64) -> SRString);
swift_rs::swift!(fn swift_get_device_is_default(index: i64) -> bool);

// =============================================================================
// Unified Audio Engine (single AVAudioEngine for both capture and monitoring)
// =============================================================================
swift_rs::swift!(fn swift_audio_engine_start(device_name: &SRString) -> bool);
swift_rs::swift!(fn swift_audio_engine_stop());
swift_rs::swift!(fn swift_audio_engine_set_device(device_name: &SRString) -> bool);
swift_rs::swift!(fn swift_audio_engine_is_running() -> bool);
swift_rs::swift!(fn swift_audio_engine_get_level() -> u8);
swift_rs::swift!(fn swift_audio_engine_start_capture() -> bool);
// Returns file path containing captured samples (or empty string on error)
swift_rs::swift!(fn swift_audio_engine_stop_capture() -> SRString);
swift_rs::swift!(fn swift_audio_engine_is_capturing() -> bool);
swift_rs::swift!(fn swift_audio_engine_get_duration_ms() -> i64);
swift_rs::swift!(fn swift_audio_engine_get_sample_count() -> i64);
swift_rs::swift!(fn swift_audio_engine_get_error() -> SRString);

/// Call the Swift hello function.
/// Returns "Hello from Swift!" to verify the interop is working.
///
/// Note: This function exists to verify the Swift-Rust FFI bridge is working.
/// It's tested in swift::tests but not called elsewhere in production code.
#[allow(dead_code)]
pub fn hello() -> String {
    unsafe { swift_hello().to_string() }
}

/// Represents an audio input device.
#[derive(Debug, Clone, PartialEq)]
pub struct SwiftAudioDevice {
    pub name: String,
    pub is_default: bool,
}

/// List all available audio input devices using AVFoundation.
///
/// Returns a vector of audio devices sorted with the default device first.
/// Returns an empty vector if no devices are available.
pub fn list_audio_devices() -> Vec<SwiftAudioDevice> {
    unsafe {
        let count = swift_refresh_audio_devices();
        let mut devices = Vec::with_capacity(count as usize);

        for i in 0..count {
            let name = swift_get_device_name(i).to_string();
            let is_default = swift_get_device_is_default(i);
            devices.push(SwiftAudioDevice { name, is_default });
        }

        devices
    }
}

/// Result of stopping audio capture.
#[derive(Debug)]
pub struct AudioCaptureStopResult {
    /// Path to the temp WAV file containing captured audio (16kHz mono)
    /// Caller should move/rename this file to the final location
    pub file_path: String,
    /// Recording duration in milliseconds
    pub duration_ms: u64,
}

// =============================================================================
// Unified Audio Engine API
// =============================================================================

/// Result of an audio engine operation.
#[derive(Debug)]
pub enum AudioEngineResult {
    /// Operation succeeded
    Ok,
    /// Operation failed with error message
    Failed(String),
}

/// Start the unified audio engine with optional device selection.
/// The engine provides level monitoring continuously once started.
///
/// # Arguments
/// * `device_name` - Optional device name; None uses the default device
pub fn audio_engine_start(device_name: Option<&str>) -> AudioEngineResult {
    unsafe {
        let success = match device_name {
            Some(name) => {
                let sr_name = SRString::from(name);
                swift_audio_engine_start(&sr_name)
            }
            None => {
                let empty = SRString::from("");
                swift_audio_engine_start(&empty)
            }
        };

        if success {
            AudioEngineResult::Ok
        } else {
            let error = swift_audio_engine_get_error().to_string();
            AudioEngineResult::Failed(if error.is_empty() {
                "Unknown error starting audio engine".to_string()
            } else {
                error
            })
        }
    }
}

/// Stop the unified audio engine.
pub fn audio_engine_stop() {
    unsafe { swift_audio_engine_stop() }
}

/// Set the audio device for the engine.
/// Engine must be running.
pub fn audio_engine_set_device(device_name: Option<&str>) -> AudioEngineResult {
    unsafe {
        let success = match device_name {
            Some(name) => {
                let sr_name = SRString::from(name);
                swift_audio_engine_set_device(&sr_name)
            }
            None => {
                let empty = SRString::from("");
                swift_audio_engine_set_device(&empty)
            }
        };

        if success {
            AudioEngineResult::Ok
        } else {
            let error = swift_audio_engine_get_error().to_string();
            AudioEngineResult::Failed(if error.is_empty() {
                "Unknown error setting audio device".to_string()
            } else {
                error
            })
        }
    }
}

/// Check if the audio engine is running.
pub fn audio_engine_is_running() -> bool {
    unsafe { swift_audio_engine_is_running() }
}

/// Get the current audio level (0-100).
/// Available whenever engine is running.
pub fn audio_engine_get_level() -> u8 {
    unsafe { swift_audio_engine_get_level() }
}

/// Start audio capture. Engine must be running.
pub fn audio_engine_start_capture() -> AudioEngineResult {
    unsafe {
        if swift_audio_engine_start_capture() {
            AudioEngineResult::Ok
        } else {
            let error = swift_audio_engine_get_error().to_string();
            AudioEngineResult::Failed(if error.is_empty() {
                "Unknown error starting capture".to_string()
            } else {
                error
            })
        }
    }
}

/// Stop audio capture and return path to the temp WAV file.
/// The Swift side writes audio to a temp WAV file to avoid dropped samples.
/// The caller should move/rename this file to the final location (instant, no I/O).
pub fn audio_engine_stop_capture() -> AudioCaptureStopResult {
    unsafe {
        let duration_ms = swift_audio_engine_get_duration_ms() as u64;
        let file_path = swift_audio_engine_stop_capture().to_string();

        if file_path.is_empty() {
            crate::warn!("No capture file path returned from Swift");
        } else {
            crate::debug!("Capture file ready: {}", file_path);
        }

        // Don't read samples here - caller will move the file directly
        // This saves ~500ms of file I/O on stop
        AudioCaptureStopResult {
            file_path,
            duration_ms,
        }
    }
}

/// Read audio samples from a WAV file.
/// Expects 16kHz mono float32 format (as written by Swift).
/// Note: No longer used since we moved to file rename approach for efficiency.
#[allow(dead_code)]
fn read_wav_samples(path: &str) -> Result<Vec<f32>, String> {
    let reader = hound::WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;

    let spec = reader.spec();
    crate::debug!(
        "WAV file spec: channels={}, sample_rate={}, bits_per_sample={}, format={:?}",
        spec.channels,
        spec.sample_rate,
        spec.bits_per_sample,
        spec.sample_format
    );

    // Read samples based on format
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader
                .into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect()
        }
        hound::SampleFormat::Int => {
            // Convert int samples to float
            let bit_depth = spec.bits_per_sample;
            let max_val = (1i32 << (bit_depth - 1)) as f32;
            reader
                .into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect()
        }
    };

    Ok(samples)
}

/// Check if currently capturing audio.
#[allow(dead_code)]
pub fn audio_engine_is_capturing() -> bool {
    unsafe { swift_audio_engine_is_capturing() }
}

/// Get the current sample count during capture.
#[allow(dead_code)]
pub fn audio_engine_get_sample_count() -> usize {
    unsafe { swift_audio_engine_get_sample_count() as usize }
}

// =============================================================================
// System Wake Notification API
// =============================================================================

/// Register a callback to be invoked when the system wakes from sleep.
/// The callback will be called on the main thread.
///
/// # Arguments
/// * `callback` - Function pointer to call on system wake
///
/// # Safety
/// The callback must be a valid function pointer that remains valid for the
/// duration of the registration. Only one callback can be registered at a time;
/// calling again replaces the previous callback.
///
/// Used by: create-wake-handler-module-for-sleep-wake-events (spec #3)
#[allow(dead_code)]
pub fn register_wake_callback(callback: WakeCallback) {
    unsafe { swift_register_wake_callback(callback as *const std::ffi::c_void) }
}

/// Unregister the wake callback and stop observing wake notifications.
/// Safe to call even if no callback is registered.
///
/// Used by: create-wake-handler-module-for-sleep-wake-events (spec #3)
#[allow(dead_code)]
pub fn unregister_wake_callback() {
    unsafe { swift_unregister_wake_callback() }
}

// =============================================================================
// Audio Device Change Notification API
// =============================================================================

/// Register a callback to be invoked when audio devices connect/disconnect.
/// The callback will be called when Core Audio detects device list changes.
///
/// # Arguments
/// * `callback` - Function pointer to call on device change
///
/// # Safety
/// The callback must be a valid function pointer that remains valid for the
/// duration of the registration. Only one callback can be registered at a time;
/// calling again replaces the previous callback.
///
/// Used by: restart-audio-engine-on-device-change spec
#[allow(dead_code)]
pub fn register_device_change_callback(callback: DeviceChangeCallback) {
    unsafe { swift_register_device_change_callback(callback as *const std::ffi::c_void) }
}

/// Unregister the device change callback and stop listening for device changes.
/// Safe to call even if no callback is registered.
///
/// Used by: restart-audio-engine-on-device-change spec
#[allow(dead_code)]
pub fn unregister_device_change_callback() {
    unsafe { swift_unregister_device_change_callback() }
}

#[cfg(test)]
#[path = "swift_test.rs"]
mod tests;
