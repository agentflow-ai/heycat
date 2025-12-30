use super::*;

#[test]
fn test_chunk_size_calculation() {
    assert_eq!(chunk_size_for_sample_rate(16000), VAD_CHUNK_SIZE_16KHZ);
    assert_eq!(chunk_size_for_sample_rate(8000), VAD_CHUNK_SIZE_8KHZ);
}

#[test]
fn test_chunk_sizes_match_formula() {
    // Verify the constants match the formula
    assert_eq!(
        VAD_CHUNK_SIZE_16KHZ,
        (DEFAULT_SAMPLE_RATE * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
    );
    assert_eq!(
        VAD_CHUNK_SIZE_8KHZ,
        (8000 * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
    );
}

#[test]
fn test_min_partial_vad_chunk_is_half_of_full_chunk() {
    // MIN_PARTIAL_VAD_CHUNK should be exactly half of VAD_CHUNK_SIZE_16KHZ
    assert_eq!(MIN_PARTIAL_VAD_CHUNK, VAD_CHUNK_SIZE_16KHZ / 2);
    assert_eq!(MIN_PARTIAL_VAD_CHUNK, 256);
}

#[test]
fn test_threshold_ordering() {
    // Wake word threshold should be lowest (most sensitive)
    assert!(VAD_THRESHOLD_WAKE_WORD < VAD_THRESHOLD_BALANCED);
    // Balanced should be in the middle
    assert!(VAD_THRESHOLD_BALANCED < VAD_THRESHOLD_SILENCE);
    // Silence threshold should be below aggressive
    assert!(VAD_THRESHOLD_SILENCE < VAD_THRESHOLD_AGGRESSIVE);
}

#[test]
fn test_sample_rate_valid_for_silero() {
    // Silero VAD only supports 8000 or 16000 Hz
    assert!(DEFAULT_SAMPLE_RATE == 16000 || DEFAULT_SAMPLE_RATE == 8000);
}

#[test]
fn test_preferred_buffer_size_reasonable() {
    // Buffer size should be a power of 2 for efficient audio processing
    assert!(PREFERRED_BUFFER_SIZE.is_power_of_two());
    // Should be at least 64 samples for stable operation
    assert!(PREFERRED_BUFFER_SIZE >= 64);
    // Should be at most 1024 to keep latency reasonable
    assert!(PREFERRED_BUFFER_SIZE <= 1024);
    // 256 is the recommended default
    assert_eq!(PREFERRED_BUFFER_SIZE, 256);
}

#[test]
fn test_buffer_latency_calculation() {
    // Verify the latency values from the doc comment
    let latency_16khz_ms = PREFERRED_BUFFER_SIZE as f32 / 16000.0 * 1000.0;
    let latency_48khz_ms = PREFERRED_BUFFER_SIZE as f32 / 48000.0 * 1000.0;

    // 256 samples @ 16kHz = 16ms
    assert!((latency_16khz_ms - 16.0).abs() < 0.1);
    // 256 samples @ 48kHz ≈ 5.3ms
    assert!((latency_48khz_ms - 5.33).abs() < 0.1);
}
