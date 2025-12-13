// StreamingTranscriber for EOU (End-of-Utterance) streaming transcription
// Processes audio in 160ms chunks (2560 samples at 16kHz)

use std::sync::mpsc::Receiver;

/// Chunk size for EOU streaming (160ms at 16kHz)
pub const CHUNK_SIZE: usize = 2560;

/// Streaming transcriber for real-time audio processing
/// Uses Parakeet EOU model for low-latency transcription
pub struct StreamingTranscriber {
    /// Audio sample receiver for streaming input
    audio_receiver: Option<Receiver<Vec<f32>>>,
    /// Buffer for accumulating audio chunks
    chunk_buffer: Vec<f32>,
}

impl Default for StreamingTranscriber {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingTranscriber {
    /// Create a new StreamingTranscriber without audio input configured
    pub fn new() -> Self {
        Self {
            audio_receiver: None,
            chunk_buffer: Vec::new(),
        }
    }

    /// Configure the audio receiver for streaming input
    pub fn with_audio_receiver(mut self, receiver: Receiver<Vec<f32>>) -> Self {
        self.audio_receiver = Some(receiver);
        self
    }

    /// Check if audio receiver is configured
    pub fn has_audio_receiver(&self) -> bool {
        self.audio_receiver.is_some()
    }

    /// Process a chunk of audio samples
    /// Accumulates samples and emits partial transcriptions when enough data is available
    ///
    /// Note: This is a stub implementation. The actual EOU model integration
    /// will be implemented in the eou-streaming-transcription spec.
    pub fn process_chunk(&mut self, samples: &[f32]) -> Option<String> {
        self.chunk_buffer.extend_from_slice(samples);

        // When we have enough samples for a chunk, process it
        if self.chunk_buffer.len() >= CHUNK_SIZE {
            // TODO: Implement EOU transcription in eou-streaming-transcription spec
            // For now, just clear the buffer and return None
            self.chunk_buffer.clear();
        }

        None
    }

    /// Clear the accumulated audio buffer
    pub fn clear_buffer(&mut self) {
        self.chunk_buffer.clear();
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> usize {
        self.chunk_buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_streaming_transcriber_new() {
        let transcriber = StreamingTranscriber::new();
        assert!(!transcriber.has_audio_receiver());
        assert_eq!(transcriber.buffer_size(), 0);
    }

    #[test]
    fn test_streaming_transcriber_default() {
        let transcriber = StreamingTranscriber::default();
        assert!(!transcriber.has_audio_receiver());
        assert_eq!(transcriber.buffer_size(), 0);
    }

    #[test]
    fn test_streaming_transcriber_with_audio_receiver() {
        let (_tx, rx) = mpsc::channel::<Vec<f32>>();
        let transcriber = StreamingTranscriber::new().with_audio_receiver(rx);
        assert!(transcriber.has_audio_receiver());
    }

    #[test]
    fn test_process_chunk_accumulates_samples() {
        let mut transcriber = StreamingTranscriber::new();
        let samples = vec![0.0f32; 1000];

        transcriber.process_chunk(&samples);
        assert_eq!(transcriber.buffer_size(), 1000);

        transcriber.process_chunk(&samples);
        assert_eq!(transcriber.buffer_size(), 2000);
    }

    #[test]
    fn test_process_chunk_clears_when_full() {
        let mut transcriber = StreamingTranscriber::new();
        let samples = vec![0.0f32; CHUNK_SIZE];

        transcriber.process_chunk(&samples);
        // Buffer should be cleared after reaching CHUNK_SIZE
        assert_eq!(transcriber.buffer_size(), 0);
    }

    #[test]
    fn test_clear_buffer() {
        let mut transcriber = StreamingTranscriber::new();
        let samples = vec![0.0f32; 1000];

        transcriber.process_chunk(&samples);
        assert_eq!(transcriber.buffer_size(), 1000);

        transcriber.clear_buffer();
        assert_eq!(transcriber.buffer_size(), 0);
    }
}
