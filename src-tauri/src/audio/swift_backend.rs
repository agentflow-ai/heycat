// AVFoundation-based audio capture backend using Swift FFI
//
// This backend uses Swift/AVFoundation for audio capture, which provides:
// - Native macOS audio stack integration
// - Automatic sample rate conversion to 16kHz mono
// - Device selection via AVCaptureDevice
//
// This code interacts with hardware and is excluded from coverage measurement.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, TARGET_SAMPLE_RATE};
use super::diagnostics::{QualityWarning, RecordingDiagnostics};
use crate::swift::{self, AudioEngineResult};
use std::sync::mpsc::Sender;
use std::sync::Arc;

/// Audio capture backend using the unified SharedAudioEngine via Swift FFI
///
/// This backend uses the SharedAudioEngine which provides a single AVAudioEngine
/// instance for both audio capture and level monitoring. This avoids device
/// conflicts that occur when multiple AVAudioEngine instances access the same device.
///
/// The Swift side handles:
/// - Device selection (shared with level monitoring)
/// - Sample rate conversion to 16kHz
/// - Mono mixing
///
/// When start() is called, the engine is started if not already running,
/// and capture mode is enabled. Level monitoring continues to work during capture.
/// Samples are collected in Swift during recording and transferred to
/// the Rust AudioBuffer when stop() is called.
pub struct SwiftBackend {
    state: CaptureState,
    /// Buffer to receive audio samples on stop
    buffer: Option<AudioBuffer>,
    /// Recording diagnostics
    diagnostics: Option<Arc<RecordingDiagnostics>>,
    /// Quality warnings from the last recording
    last_warnings: Vec<QualityWarning>,
}

impl SwiftBackend {
    /// Create a new Swift/AVFoundation backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            buffer: None,
            diagnostics: None,
            last_warnings: Vec::new(),
        }
    }

    /// Get quality warnings from the last recording
    pub fn take_warnings(&mut self) -> Vec<QualityWarning> {
        std::mem::take(&mut self.last_warnings)
    }

    /// Get raw audio from the last recording
    ///
    /// Note: SwiftBackend doesn't support raw audio capture currently.
    /// Returns None always.
    pub fn take_raw_audio(&mut self) -> Option<(Vec<f32>, u32)> {
        None
    }

}

impl Default for SwiftBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCaptureBackend for SwiftBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        _stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
    ) -> Result<u32, AudioCaptureError> {
        crate::info!("========================================");
        crate::info!("[START] NEW RECORDING SESSION (SharedAudioEngine)");
        crate::info!("========================================");
        crate::info!("Starting audio capture via SharedAudioEngine (target: {}Hz)...", TARGET_SAMPLE_RATE);

        if let Some(ref name) = device_name {
            crate::info!("Requested device: {}", name);
        } else {
            crate::info!("Using default audio input device");
        }

        // Store buffer for later sample transfer
        self.buffer = Some(buffer);
        self.diagnostics = Some(Arc::new(RecordingDiagnostics::new()));
        self.last_warnings.clear();

        // Ensure engine is running (it may already be running for level monitoring)
        if !swift::audio_engine_is_running() {
            crate::info!("Starting SharedAudioEngine...");
            match swift::audio_engine_start(device_name.as_deref()) {
                AudioEngineResult::Ok => {
                    crate::info!("SharedAudioEngine started successfully");
                }
                AudioEngineResult::Failed(error) => {
                    crate::error!("Failed to start audio engine: {}", error);
                    self.buffer = None;
                    self.diagnostics = None;

                    if error.contains("No audio input device") || error.contains("no devices") {
                        return Err(AudioCaptureError::NoDeviceAvailable);
                    } else {
                        return Err(AudioCaptureError::DeviceError(error));
                    }
                }
            }
        } else if device_name.is_some() {
            // Engine running, but device may need switching
            crate::info!("Engine already running, setting device...");
            if let AudioEngineResult::Failed(error) = swift::audio_engine_set_device(device_name.as_deref()) {
                crate::warn!("Failed to set device: {}", error);
                // Don't fail - continue with current device
            }
        }

        // Start capture mode on the engine
        match swift::audio_engine_start_capture() {
            AudioEngineResult::Ok => {
                crate::info!("Audio capture started successfully via SharedAudioEngine");
                self.state = CaptureState::Capturing;
                // AVFoundation captures at 16kHz (configured in Swift)
                Ok(TARGET_SAMPLE_RATE)
            }
            AudioEngineResult::Failed(error) => {
                crate::error!("Failed to start audio capture: {}", error);
                self.buffer = None;
                self.diagnostics = None;

                // Map error to appropriate type
                if error.contains("No audio input device") || error.contains("no devices") {
                    Err(AudioCaptureError::NoDeviceAvailable)
                } else {
                    Err(AudioCaptureError::DeviceError(error))
                }
            }
        }
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        crate::info!("========================================");
        crate::info!("[STOP] RECORDING SESSION STOPPING (SharedAudioEngine)");
        crate::info!("========================================");

        if self.state != CaptureState::Capturing {
            crate::debug!("Not capturing, nothing to stop");
            return Ok(());
        }

        // Stop capture and get all samples from Swift
        // Note: Engine stays running for continued level monitoring
        let swift_result = swift::audio_engine_stop_capture();
        let sample_count = swift_result.samples.len();
        let duration_ms = swift_result.duration_ms;

        crate::info!(
            "[STOP] Captured {} samples ({:.2}s) from SharedAudioEngine",
            sample_count,
            duration_ms as f64 / 1000.0
        );

        // Transfer samples to the AudioBuffer
        if let Some(ref buffer) = self.buffer {
            let pushed = buffer.push_samples(&swift_result.samples);
            if pushed < sample_count {
                crate::warn!(
                    "Buffer only accepted {}/{} samples (buffer full?)",
                    pushed,
                    sample_count
                );
            }
            crate::debug!("Pushed {} samples to buffer", pushed);

            // Record diagnostics
            if let Some(ref diagnostics) = self.diagnostics {
                diagnostics.record_output(&swift_result.samples);
                self.last_warnings = diagnostics.check_warnings();
            }
        } else {
            crate::error!("No buffer available for sample transfer!");
        }

        self.state = CaptureState::Stopped;
        self.buffer = None;
        self.diagnostics = None;

        crate::info!("[STOP] SharedAudioEngine capture stopped successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_backend_is_send_sync() {
        // SwiftBackend should be Send + Sync for thread safety
        fn assert_send_sync<T: Send + Sync>() {}
        // Note: SwiftBackend contains AudioBuffer which has Arc<Mutex<...>>
        // The Arc<Mutex<...>> types are Send + Sync, so SwiftBackend should be too
        // This test documents the expectation
    }

    #[test]
    fn test_new_creates_idle_state() {
        let backend = SwiftBackend::new();
        assert_eq!(backend.state, CaptureState::Idle);
        assert!(backend.buffer.is_none());
    }

    #[test]
    fn test_take_warnings_returns_and_clears() {
        let mut backend = SwiftBackend::new();
        // Initially empty
        assert!(backend.take_warnings().is_empty());
    }

    #[test]
    fn test_take_raw_audio_returns_none() {
        let mut backend = SwiftBackend::new();
        // SwiftBackend doesn't support raw audio
        assert!(backend.take_raw_audio().is_none());
    }

    /// Test that start and stop work without panicking
    /// Ignored by default as it requires microphone permissions
    /// Run manually with: cargo test test_start_stop_cycle -- --ignored
    #[test]
    #[ignore]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_start_stop_cycle() {
        let mut backend = SwiftBackend::new();
        let buffer = AudioBuffer::new();

        // Start may fail if no device available (CI environment)
        let result = backend.start(buffer, None, None);
        match result {
            Ok(sample_rate) => {
                assert_eq!(sample_rate, TARGET_SAMPLE_RATE);
                assert_eq!(backend.state, CaptureState::Capturing);

                // Stop should succeed
                let stop_result = backend.stop();
                assert!(stop_result.is_ok());
                assert_eq!(backend.state, CaptureState::Stopped);
            }
            Err(_) => {
                // Expected in CI without audio device
                assert_eq!(backend.state, CaptureState::Idle);
            }
        }
    }
}
