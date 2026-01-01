// Tests for the transcription storage module

// TranscriptionStorage requires a TursoClient and AppHandle for testing.
// The actual storage behavior is tested through integration tests.
// Here we just verify the module compiles correctly.

#[test]
fn test_transcription_storage_is_sized() {
    // Verify TranscriptionStorage can be used with sized bounds
    fn _requires_sized<T: Sized>() {}
    _requires_sized::<super::TranscriptionStorage>();
}
