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
    /// Buffer to receive audio samples on stop (kept for API compatibility but no longer used)
    buffer: Option<AudioBuffer>,
    /// Recording diagnostics
    diagnostics: Option<Arc<RecordingDiagnostics>>,
    /// Quality warnings from the last recording
    last_warnings: Vec<QualityWarning>,
    /// Path to the captured WAV file from the last recording
    last_capture_file_path: Option<String>,
    /// Duration of the last recording in milliseconds
    last_duration_ms: u64,
}

impl SwiftBackend {
    /// Create a new Swift/AVFoundation backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            buffer: None,
            diagnostics: None,
            last_warnings: Vec::new(),
            last_capture_file_path: None,
            last_duration_ms: 0,
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

    /// Take the capture file path from the last recording
    ///
    /// Returns the path to the temp WAV file and duration. The caller should
    /// move/rename this file to the final location (instant, no I/O).
    pub fn take_capture_file(&mut self) -> Option<(String, u64)> {
        self.last_capture_file_path.take().map(|path| (path, self.last_duration_ms))
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

        // Engine should already be running (pre-initialized at app startup)
        // This is a defensive fallback in case initialization failed or was skipped
        if !swift::audio_engine_is_running() {
            crate::warn!("Engine not pre-initialized, starting now (lazy initialization fallback)");
            match swift::audio_engine_start(device_name.as_deref()) {
                AudioEngineResult::Ok => {
                    crate::info!("SharedAudioEngine started successfully (lazy)");
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
            // Engine already running (expected path), handle device switching if needed
            crate::info!("Engine already running, setting device...");
            if let AudioEngineResult::Failed(error) = swift::audio_engine_set_device(device_name.as_deref()) {
                crate::warn!("Failed to set device: {}", error);
                // Don't fail - continue with current device
            }
        } else {
            // Engine running with default device - ideal path after pre-initialization
            crate::info!("Engine already running with default device");
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

        // Stop capture and get file path from Swift
        // Note: Engine stays running for continued level monitoring
        // Note: We don't read the file here - caller will move it directly (instant, no I/O)
        let swift_result = swift::audio_engine_stop_capture();
        let duration_ms = swift_result.duration_ms;

        crate::info!(
            "[STOP] Capture complete ({:.2}s), file ready at: {}",
            duration_ms as f64 / 1000.0,
            swift_result.file_path
        );

        // Store file path and duration for caller to retrieve via take_capture_file()
        self.last_capture_file_path = if swift_result.file_path.is_empty() {
            None
        } else {
            Some(swift_result.file_path)
        };
        self.last_duration_ms = duration_ms;

        // Diagnostics skipped - we no longer have samples in memory
        // Warnings will be empty, which is fine for the optimized path
        self.last_warnings.clear();

        self.state = CaptureState::Stopped;
        self.buffer = None;
        self.diagnostics = None;

        crate::info!("[STOP] SharedAudioEngine capture stopped successfully");
        Ok(())
    }
}

#[cfg(test)]
#[path = "swift_backend_test.rs"]
mod tests;
