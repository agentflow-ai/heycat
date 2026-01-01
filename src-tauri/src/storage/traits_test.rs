// Tests for the storage traits

// Trait tests verify the trait definitions compile correctly.
// Actual implementations are tested in the turso module.

#[test]
fn test_recording_store_backend_is_object_safe() {
    // Verify the trait is object safe (can be used with dyn)
    fn _takes_dyn(_: &dyn super::RecordingStoreBackend) {}
}

#[test]
fn test_transcription_store_backend_is_object_safe() {
    // Verify the trait is object safe (can be used with dyn)
    fn _takes_dyn(_: &dyn super::TranscriptionStoreBackend) {}
}
