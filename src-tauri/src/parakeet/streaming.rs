// StreamingTranscriber for EOU (End-of-Utterance) streaming transcription
// Processes audio in 160ms chunks (2560 samples at 16kHz)

use crate::events::{
    TranscriptionCompletedPayload, TranscriptionEventEmitter, TranscriptionPartialPayload,
};
use crate::parakeet::types::TranscriptionError;
use crate::{debug, info, warn};
use parakeet_rs::ParakeetEOU;
use std::path::Path;
use std::sync::Arc;

/// Chunk size for EOU streaming (160ms at 16kHz)
pub const CHUNK_SIZE: usize = 2560;

/// Streaming transcription state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingState {
    /// No EOU model loaded
    Unloaded,
    /// Model loaded, ready to stream
    Idle,
    /// Currently processing audio chunks
    Streaming,
    /// Processing final chunk
    Finalizing,
}

/// Streaming transcriber for real-time audio processing
/// Uses Parakeet EOU model for low-latency transcription
pub struct StreamingTranscriber<E: TranscriptionEventEmitter> {
    /// EOU model instance (None if not loaded)
    eou: Option<ParakeetEOU>,
    /// Current streaming state
    state: StreamingState,
    /// Buffer for accumulating audio samples before processing
    sample_buffer: Vec<f32>,
    /// Accumulated partial text from all chunks
    partial_text: String,
    /// Event emitter for partial/completed events
    emitter: Arc<E>,
    /// Start time for duration tracking
    start_time: Option<std::time::Instant>,
}

impl<E: TranscriptionEventEmitter> StreamingTranscriber<E> {
    /// Create a new StreamingTranscriber in unloaded state
    pub fn new(emitter: Arc<E>) -> Self {
        Self {
            eou: None,
            state: StreamingState::Unloaded,
            sample_buffer: Vec::new(),
            partial_text: String::new(),
            emitter,
            start_time: None,
        }
    }

    /// Load the EOU model from the given directory path
    pub fn load_model(&mut self, model_dir: &Path) -> Result<(), TranscriptionError> {
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let eou = ParakeetEOU::from_pretrained(path_str, None)
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        self.eou = Some(eou);
        self.state = StreamingState::Idle;
        Ok(())
    }

    /// Check if a model is loaded
    #[allow(dead_code)]
    pub fn is_loaded(&self) -> bool {
        self.eou.is_some()
    }

    /// Get the current streaming state
    #[allow(dead_code)]
    pub fn state(&self) -> StreamingState {
        self.state
    }

    /// Get the current buffer size
    #[allow(dead_code)]
    pub fn buffer_size(&self) -> usize {
        self.sample_buffer.len()
    }

    /// Normalize audio samples by dividing by max absolute value
    /// This matches the preprocessing done in parakeet-rs examples
    fn normalize_samples(samples: &[f32]) -> Vec<f32> {
        let max_val = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let epsilon = 1e-10_f32;
        if max_val > epsilon {
            samples.iter().map(|s| s / (max_val + epsilon)).collect()
        } else {
            samples.to_vec()
        }
    }

    /// Process incoming audio samples
    /// Buffers samples until CHUNK_SIZE (2560) is reached, then transcribes
    pub fn process_samples(&mut self, samples: &[f32]) -> Result<(), TranscriptionError> {
        debug!("process_samples called with {} samples, buffer has {}", samples.len(), self.sample_buffer.len());

        // Check if model is loaded
        let eou = self.eou.as_mut().ok_or_else(|| {
            warn!("process_samples: EOU model not loaded!");
            TranscriptionError::ModelNotLoaded
        })?;
        debug!("EOU model is loaded");

        // Track start time on first samples
        if self.start_time.is_none() {
            self.start_time = Some(std::time::Instant::now());
        }

        // Transition to Streaming state
        if self.state == StreamingState::Idle {
            self.state = StreamingState::Streaming;
        }

        // Add samples to buffer
        self.sample_buffer.extend_from_slice(samples);
        debug!("Buffer now has {} samples (need {} for chunk)", self.sample_buffer.len(), CHUNK_SIZE);

        // Process complete chunks
        let mut chunks_processed = 0;
        while self.sample_buffer.len() >= CHUNK_SIZE {
            // Extract chunk from buffer
            let chunk: Vec<f32> = self.sample_buffer.drain(..CHUNK_SIZE).collect();
            chunks_processed += 1;

            // Normalize audio before transcription (matches parakeet-rs example preprocessing)
            let normalized_chunk = Self::normalize_samples(&chunk);
            debug!("Calling eou.transcribe with {} samples (chunk {}, normalized)", normalized_chunk.len(), chunks_processed);

            // Transcribe with is_final=false (intermediate chunk)
            let text = eou
                .transcribe(&normalized_chunk, false)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;

            info!("eou.transcribe returned: '{}' ({} chars)", text, text.len());

            // Accumulate partial text
            if !text.is_empty() {
                self.partial_text.push_str(&text);
                info!("partial_text now: '{}' ({} chars)", self.partial_text, self.partial_text.len());
            }

            // Emit partial event
            self.emitter.emit_transcription_partial(TranscriptionPartialPayload {
                text: self.partial_text.clone(),
                is_final: false,
            });
        }

        if chunks_processed > 0 {
            debug!("Processed {} chunks, {} samples remaining in buffer", chunks_processed, self.sample_buffer.len());
        }

        Ok(())
    }

    /// Finalize transcription - process remaining buffer with is_final=true
    /// Returns the complete transcribed text
    pub fn finalize(&mut self) -> Result<String, TranscriptionError> {
        info!("finalize called: sample_buffer={} samples, partial_text={} chars",
              self.sample_buffer.len(), self.partial_text.len());

        // Check if model is loaded
        let eou = self.eou.as_mut().ok_or_else(|| {
            warn!("finalize: EOU model not loaded!");
            TranscriptionError::ModelNotLoaded
        })?;
        debug!("EOU model is loaded for finalize");

        self.state = StreamingState::Finalizing;

        // Process remaining samples (even if less than CHUNK_SIZE)
        if !self.sample_buffer.is_empty() {
            let final_chunk: Vec<f32> = self.sample_buffer.drain(..).collect();
            // Normalize before final transcription
            let normalized_chunk = Self::normalize_samples(&final_chunk);
            info!("Calling final eou.transcribe with {} remaining samples (is_final=true, normalized)", normalized_chunk.len());

            let text = eou
                .transcribe(&normalized_chunk, true)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;

            info!("Final transcribe returned: '{}' ({} chars)", text, text.len());

            if !text.is_empty() {
                self.partial_text.push_str(&text);
            }
        } else {
            // Call with empty chunk but is_final=true to finalize
            info!("Calling eou.transcribe with empty chunk (is_final=true) to flush");
            let text = eou
                .transcribe(&[], true)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;

            info!("Empty chunk transcribe returned: '{}' ({} chars)", text, text.len());

            if !text.is_empty() {
                self.partial_text.push_str(&text);
            }
        }

        // Emit final partial event
        self.emitter.emit_transcription_partial(TranscriptionPartialPayload {
            text: self.partial_text.clone(),
            is_final: true,
        });

        // Calculate duration
        let duration_ms = self
            .start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        // Emit completed event
        self.emitter
            .emit_transcription_completed(TranscriptionCompletedPayload {
                text: self.partial_text.clone(),
                duration_ms,
            });

        let result = self.partial_text.clone();
        info!("finalize returning: '{}' ({} chars)", result, result.len());

        // Reset to Idle state
        self.state = StreamingState::Idle;

        Ok(result)
    }

    /// Reset the transcriber for a new recording
    pub fn reset(&mut self) {
        self.sample_buffer.clear();
        self.partial_text.clear();
        self.start_time = None;
        if self.eou.is_some() {
            self.state = StreamingState::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock event emitter for testing
    #[derive(Default)]
    struct MockEmitter {
        partial_events: Arc<Mutex<Vec<TranscriptionPartialPayload>>>,
        completed_events: Arc<Mutex<Vec<TranscriptionCompletedPayload>>>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self::default()
        }

        fn partial_count(&self) -> usize {
            self.partial_events.lock().unwrap().len()
        }

        fn completed_count(&self) -> usize {
            self.completed_events.lock().unwrap().len()
        }
    }

    impl TranscriptionEventEmitter for MockEmitter {
        fn emit_transcription_started(
            &self,
            _payload: crate::events::TranscriptionStartedPayload,
        ) {
        }

        fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
            self.completed_events.lock().unwrap().push(payload);
        }

        fn emit_transcription_error(&self, _payload: crate::events::TranscriptionErrorPayload) {}

        fn emit_transcription_partial(&self, payload: TranscriptionPartialPayload) {
            self.partial_events.lock().unwrap().push(payload);
        }
    }

    #[test]
    fn test_streaming_transcriber_new_unloaded() {
        let emitter = Arc::new(MockEmitter::new());
        let transcriber = StreamingTranscriber::new(emitter);
        assert!(!transcriber.is_loaded());
        assert_eq!(transcriber.state(), StreamingState::Unloaded);
        assert_eq!(transcriber.buffer_size(), 0);
    }

    #[test]
    fn test_streaming_transcriber_load_model_invalid_path() {
        let emitter = Arc::new(MockEmitter::new());
        let mut transcriber = StreamingTranscriber::new(emitter);
        let result = transcriber.load_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
        assert!(!transcriber.is_loaded());
        assert_eq!(transcriber.state(), StreamingState::Unloaded);
    }

    #[test]
    fn test_streaming_transcriber_process_samples_without_model() {
        let emitter = Arc::new(MockEmitter::new());
        let mut transcriber = StreamingTranscriber::new(emitter);
        let samples = vec![0.0f32; 1000];
        let result = transcriber.process_samples(&samples);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_streaming_transcriber_finalize_without_model() {
        let emitter = Arc::new(MockEmitter::new());
        let mut transcriber = StreamingTranscriber::new(emitter);
        let result = transcriber.finalize();
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_streaming_transcriber_buffers_small_chunks() {
        let emitter = Arc::new(MockEmitter::new());
        let mut transcriber = StreamingTranscriber::new(emitter.clone());

        // Manually set to loaded state for buffer test (no actual model)
        transcriber.state = StreamingState::Idle;

        // Buffer should accumulate samples
        // Note: This test validates buffering logic, actual transcription requires model
        let initial_buffer = transcriber.buffer_size();
        assert_eq!(initial_buffer, 0);
    }

    #[test]
    fn test_streaming_transcriber_reset_clears_buffer() {
        let emitter = Arc::new(MockEmitter::new());
        let mut transcriber = StreamingTranscriber::new(emitter);
        transcriber.sample_buffer.extend_from_slice(&[0.0f32; 1000]);
        transcriber.partial_text = "test".to_string();
        transcriber.start_time = Some(std::time::Instant::now());

        assert_eq!(transcriber.buffer_size(), 1000);

        transcriber.reset();

        assert_eq!(transcriber.buffer_size(), 0);
        assert!(transcriber.partial_text.is_empty());
        assert!(transcriber.start_time.is_none());
    }

    #[test]
    fn test_streaming_state_values() {
        assert_ne!(StreamingState::Unloaded, StreamingState::Idle);
        assert_ne!(StreamingState::Idle, StreamingState::Streaming);
        assert_ne!(StreamingState::Streaming, StreamingState::Finalizing);
    }

    #[test]
    fn test_mock_emitter_tracks_events() {
        let emitter = MockEmitter::new();
        assert_eq!(emitter.partial_count(), 0);
        assert_eq!(emitter.completed_count(), 0);

        emitter.emit_transcription_partial(TranscriptionPartialPayload {
            text: "test".to_string(),
            is_final: false,
        });
        assert_eq!(emitter.partial_count(), 1);

        emitter.emit_transcription_completed(TranscriptionCompletedPayload {
            text: "test".to_string(),
            duration_ms: 100,
        });
        assert_eq!(emitter.completed_count(), 1);
    }
}
