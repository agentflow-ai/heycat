//! Swift FFI bindings module.
//!
//! This module provides safe Rust wrappers around Swift functions
//! compiled via swift-rs.

use swift_rs::SRString;

// Define the FFI function signatures
// SRString is the Swift-Rs string type that can be safely passed across FFI

// Hello function for testing
swift_rs::swift!(fn swift_hello() -> SRString);

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
    /// Captured audio samples at 16kHz mono
    pub samples: Vec<f32>,
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

/// Stop audio capture and retrieve captured samples from temp file.
/// The Swift side writes audio to a temp WAV file to avoid dropped samples.
pub fn audio_engine_stop_capture() -> AudioCaptureStopResult {
    unsafe {
        let duration_ms = swift_audio_engine_get_duration_ms() as u64;
        let file_path = swift_audio_engine_stop_capture().to_string();

        // Read samples from temp WAV file
        let samples = if !file_path.is_empty() {
            match read_wav_samples(&file_path) {
                Ok(s) => {
                    crate::debug!("Read {} samples from capture file", s.len());
                    s
                }
                Err(e) => {
                    crate::error!("Failed to read capture file {}: {}", file_path, e);
                    Vec::new()
                }
            }
        } else {
            crate::warn!("No capture file path returned from Swift");
            Vec::new()
        };

        // Clean up temp file
        if !file_path.is_empty() {
            if let Err(e) = std::fs::remove_file(&file_path) {
                crate::warn!("Failed to remove temp capture file: {}", e);
            }
        }

        AudioCaptureStopResult {
            samples,
            duration_ms,
        }
    }
}

/// Read audio samples from a WAV file.
/// Expects 16kHz mono float32 format (as written by Swift).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_hello() {
        let result = hello();
        assert_eq!(result, "Hello from Swift!");
    }

    #[test]
    fn test_list_audio_devices_returns_vec() {
        let devices = list_audio_devices();
        // Should return a vector (may be empty if no devices)
        // If devices exist, default should be first
        if !devices.is_empty() && devices.iter().any(|d| d.is_default) {
            assert!(devices[0].is_default, "Default device should be first");
        }
    }

    /// Test audio engine start/capture/stop cycle
    /// Ignored by default as it requires:
    /// - Microphone permissions granted to the test process
    /// - An actual audio input device
    /// Run manually with: cargo test test_audio_engine_capture -- --ignored
    #[test]
    #[ignore]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_audio_engine_capture() {
        // Start engine (may fail if no audio device available)
        let result = audio_engine_start(None);

        match result {
            AudioEngineResult::Ok => {
                // Start capture
                match audio_engine_start_capture() {
                    AudioEngineResult::Ok => {
                        // Capture a tiny bit
                        std::thread::sleep(std::time::Duration::from_millis(100));

                        // Stop capture and get samples
                        let stop_result = audio_engine_stop_capture();

                        // Duration should be approximately 100ms
                        assert!(stop_result.duration_ms >= 50, "Duration should be at least 50ms");
                        assert!(stop_result.duration_ms <= 500, "Duration should be at most 500ms");
                    }
                    AudioEngineResult::Failed(_) => {
                        // Expected in CI without audio device
                    }
                }

                // Stop engine
                audio_engine_stop();
            }
            AudioEngineResult::Failed(_) => {
                // Expected in CI without audio device
            }
        }
    }

    #[test]
    fn test_audio_engine_is_running_query() {
        // Ensure clean state by stopping any running engine
        audio_engine_stop();

        // After stopping, should not be running
        assert!(!audio_engine_is_running(), "Engine should not be running after stop");
    }
}
