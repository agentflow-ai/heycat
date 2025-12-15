// Circular buffer for wake word audio window
// Maintains a fixed-size rolling window of audio samples for analysis

/// A fixed-size circular buffer for audio samples
///
/// Used to maintain a rolling window of audio data (~1-2 seconds)
/// for wake word detection analysis. When full, oldest samples
/// are overwritten by new samples.
#[derive(Debug)]
pub struct CircularBuffer {
    /// Internal storage for samples
    data: Vec<f32>,
    /// Maximum capacity (number of samples)
    capacity: usize,
    /// Write position (next index to write to)
    write_pos: usize,
    /// Number of samples currently in buffer
    len: usize,
    /// Total samples ever pushed (monotonic counter for tracking analyzed audio)
    total_samples_pushed: u64,
}

impl CircularBuffer {
    /// Create a new circular buffer with the given capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of f32 samples to store
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            len: 0,
            total_samples_pushed: 0,
        }
    }

    /// Create a buffer sized for a specific duration at a given sample rate
    ///
    /// # Arguments
    /// * `duration_secs` - Duration in seconds
    /// * `sample_rate` - Sample rate in Hz
    pub fn for_duration(duration_secs: f32, sample_rate: u32) -> Self {
        let capacity = (duration_secs * sample_rate as f32) as usize;
        Self::new(capacity)
    }

    /// Push samples into the buffer
    ///
    /// If the buffer is full, oldest samples are overwritten.
    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.data[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            if self.len < self.capacity {
                self.len += 1;
            }
        }
        self.total_samples_pushed += samples.len() as u64;
    }

    /// Get all samples in chronological order
    ///
    /// Returns samples from oldest to newest.
    pub fn get_samples(&self) -> Vec<f32> {
        if self.len < self.capacity {
            // Buffer not yet full - samples are at start
            self.data[..self.len].to_vec()
        } else {
            // Buffer full - need to unwrap from write position
            let mut result = Vec::with_capacity(self.capacity);
            result.extend_from_slice(&self.data[self.write_pos..]);
            result.extend_from_slice(&self.data[..self.write_pos]);
            result
        }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        crate::trace!("[buffer] Buffer cleared, was holding {} samples", self.len);
        self.write_pos = 0;
        self.len = 0;
        // Note: total_samples_pushed is NOT reset here - it's reset separately
        // when analysis tracking needs to be reset (via reset_sample_counter)
    }

    /// Get the total number of samples ever pushed to this buffer
    ///
    /// This is a monotonic counter used for tracking which audio has been analyzed.
    pub fn total_samples_pushed(&self) -> u64 {
        self.total_samples_pushed
    }

    /// Reset the total samples counter (call when starting fresh analysis)
    pub fn reset_sample_counter(&mut self) {
        self.total_samples_pushed = 0;
    }

    /// Get the current number of samples in the buffer
    #[allow(dead_code)] // Used in tests and for debugging
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if the buffer is full
    #[allow(dead_code)] // Used in tests and for debugging
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Get the capacity of the buffer
    #[allow(dead_code)] // Used in tests and for debugging
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buffer = CircularBuffer::new(100);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 100);
    }

    #[test]
    fn test_for_duration_calculates_capacity() {
        // 1 second at 16000 Hz = 16000 samples
        let buffer = CircularBuffer::for_duration(1.0, 16000);
        assert_eq!(buffer.capacity(), 16000);

        // 2 seconds at 16000 Hz = 32000 samples
        let buffer = CircularBuffer::for_duration(2.0, 16000);
        assert_eq!(buffer.capacity(), 32000);
    }

    #[test]
    fn test_push_samples_increases_length() {
        let mut buffer = CircularBuffer::new(100);
        buffer.push_samples(&[1.0, 2.0, 3.0]);
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_push_samples_wraps_when_full() {
        let mut buffer = CircularBuffer::new(5);
        buffer.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(buffer.is_full());

        // Push more samples, should overwrite oldest
        buffer.push_samples(&[6.0, 7.0]);
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 5);

        // Should have [3.0, 4.0, 5.0, 6.0, 7.0] in chronological order
        let samples = buffer.get_samples();
        assert_eq!(samples, vec![3.0, 4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_get_samples_returns_chronological_order() {
        let mut buffer = CircularBuffer::new(5);
        buffer.push_samples(&[1.0, 2.0, 3.0]);
        let samples = buffer.get_samples();
        assert_eq!(samples, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_get_samples_handles_wrap_around() {
        let mut buffer = CircularBuffer::new(4);
        buffer.push_samples(&[1.0, 2.0, 3.0, 4.0]); // Full: [1, 2, 3, 4]
        buffer.push_samples(&[5.0, 6.0]); // Wrapped: [5, 6, 3, 4] with write_pos=2

        let samples = buffer.get_samples();
        assert_eq!(samples, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_clear_resets_buffer() {
        let mut buffer = CircularBuffer::new(5);
        buffer.push_samples(&[1.0, 2.0, 3.0]);
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_total_samples_pushed_counter() {
        let mut buffer = CircularBuffer::new(5);
        assert_eq!(buffer.total_samples_pushed(), 0);

        buffer.push_samples(&[1.0, 2.0, 3.0]);
        assert_eq!(buffer.total_samples_pushed(), 3);

        buffer.push_samples(&[4.0, 5.0]);
        assert_eq!(buffer.total_samples_pushed(), 5);

        // Counter continues even when buffer wraps
        buffer.push_samples(&[6.0, 7.0, 8.0]);
        assert_eq!(buffer.total_samples_pushed(), 8);

        // clear() doesn't reset the counter
        buffer.clear();
        assert_eq!(buffer.total_samples_pushed(), 8);

        // reset_sample_counter() does reset it
        buffer.reset_sample_counter();
        assert_eq!(buffer.total_samples_pushed(), 0);
    }

    #[test]
    fn test_empty_buffer_returns_empty_samples() {
        let buffer = CircularBuffer::new(100);
        assert!(buffer.get_samples().is_empty());
    }
}
