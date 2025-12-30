// Tests for dictionary command logic
// Note: The actual Tauri commands require an AppHandle which is difficult to mock.
// These tests focus on the validation and error mapping logic that can be tested in isolation.
// Integration testing is done via the frontend E2E tests.

use super::*;

#[test]
fn test_to_user_error_not_found() {
    let error = DictionaryError::NotFound("abc123".to_string());
    let message = to_user_error(error);
    assert!(message.contains("abc123"));
    assert!(message.contains("not found"));
}

#[test]
fn test_to_user_error_duplicate() {
    let error = DictionaryError::DuplicateId("abc123".to_string());
    let message = to_user_error(error);
    assert!(message.contains("abc123"));
    assert!(message.contains("already exists"));
}

#[test]
fn test_to_user_error_persistence() {
    let error = DictionaryError::PersistenceError("disk full".to_string());
    let message = to_user_error(error);
    assert!(message.contains("save"));
    assert!(message.contains("disk full"));
}

#[test]
fn test_to_user_error_load() {
    let error = DictionaryError::LoadError("corrupt file".to_string());
    let message = to_user_error(error);
    assert!(message.contains("load"));
    assert!(message.contains("corrupt file"));
}
