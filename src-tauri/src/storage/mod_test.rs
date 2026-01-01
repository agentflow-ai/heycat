// Tests for the storage module's public API

use super::*;

#[test]
fn test_recording_storage_exported() {
    // Verify RecordingStorage is exported through the module
    fn _takes_storage(_: &RecordingStorage) {}
}

#[test]
fn test_window_context_exported() {
    // Verify WindowContext is exported through the module
    fn _takes_ctx(_: &WindowContext) {}
}

#[test]
fn test_store_recording_function_exported() {
    // Verify store_recording is exported
    fn _takes_fn(_: fn(&tauri::AppHandle, &crate::recording::RecordingMetadata, &str)) {}
    _takes_fn(store_recording);
}

#[test]
fn test_store_transcription_function_exported() {
    // Verify store_transcription is exported
    fn _takes_fn(_: fn(&tauri::AppHandle, &str, &str, u64)) {}
    _takes_fn(store_transcription);
}
