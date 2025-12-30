use super::*;

// Tests removed per docs/TESTING.md:
// - test_shared_model_new_is_unloaded: Obvious default
// - test_shared_model_default_is_unloaded: Obvious default (duplicate)
// - test_shared_model_is_clone: Type system guarantee (Arc semantics)
// - test_concurrent_access_does_not_panic: Rust type system guarantees Send+Sync

// ==================== Behavior Tests ====================
// These test actual user-visible behavior and error handling

#[test]
fn test_transcribe_file_returns_error_for_nonexistent_file() {
    // With WAV validation, nonexistent files are caught BEFORE the model check
    // This is intentional - we want to fail fast on invalid input
    let model = SharedTranscriptionModel::new();
    let result = model.transcribe_file("/nonexistent/audio.wav");
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
}

#[test]
fn test_transcribe_file_returns_error_for_empty_path() {
    let model = SharedTranscriptionModel::new();
    let result = model.transcribe_file("");
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
}

#[test]
fn test_transcribe_samples_returns_error_when_model_not_loaded() {
    let model = SharedTranscriptionModel::new();
    let result = model.transcribe_samples(vec![0.1, 0.2, 0.3], 16000, 1);
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
}

#[test]
fn test_transcribe_samples_returns_error_for_empty_samples() {
    let model = SharedTranscriptionModel::new();
    let result = model.transcribe_samples(vec![], 16000, 1);
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
}

#[test]
fn test_load_fails_with_invalid_path() {
    let model = SharedTranscriptionModel::new();
    let result = model.load(Path::new("/nonexistent/path/to/model"));
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
}

// ==================== State Machine Tests ====================
// These test the complete state transition workflow

#[test]
fn test_reset_to_idle_state_transitions() {
    let model = SharedTranscriptionModel::new();

    // Test reset from Completed
    {
        let mut state = model.state.lock();
        *state = TranscriptionState::Completed;
    }
    model.reset_to_idle().unwrap();
    assert_eq!(model.state(), TranscriptionState::Idle);

    // Test reset from Error
    {
        let mut state = model.state.lock();
        *state = TranscriptionState::Error;
    }
    model.reset_to_idle().unwrap();
    assert_eq!(model.state(), TranscriptionState::Idle);

    // Test noop from Unloaded (doesn't transition)
    {
        let mut state = model.state.lock();
        *state = TranscriptionState::Unloaded;
    }
    model.reset_to_idle().unwrap();
    assert_eq!(model.state(), TranscriptionState::Unloaded);
}

// ==================== TranscribingGuard Tests ====================
// RAII guard is critical for panic safety - these are behavior tests

#[test]
fn test_guard_state_lifecycle() {
    let state = Arc::new(Mutex::new(TranscriptionState::Idle));

    // Test normal lifecycle: Idle -> Transcribing -> Idle
    {
        let _guard = TranscribingGuard::new(state.clone()).unwrap();
        assert_eq!(*state.lock(), TranscriptionState::Transcribing);
    }
    assert_eq!(*state.lock(), TranscriptionState::Idle);

    // Test complete_success: stays Completed after drop
    {
        let mut guard = TranscribingGuard::new(state.clone()).unwrap();
        guard.complete_success();
        assert_eq!(*state.lock(), TranscriptionState::Completed);
    }
    assert_eq!(*state.lock(), TranscriptionState::Completed);

    // Reset for next test
    *state.lock() = TranscriptionState::Idle;

    // Test complete_with_error: stays Error after drop
    {
        let mut guard = TranscribingGuard::new(state.clone()).unwrap();
        guard.complete_with_error();
        assert_eq!(*state.lock(), TranscriptionState::Error);
    }
    assert_eq!(*state.lock(), TranscriptionState::Error);
}

#[test]
fn test_guard_resets_state_to_idle_on_panic() {
    use std::panic::{self, AssertUnwindSafe};

    let state = Arc::new(Mutex::new(TranscriptionState::Idle));
    let state_clone = state.clone();

    // AssertUnwindSafe is needed because parking_lot::Mutex doesn't implement
    // UnwindSafe by default. This is safe here because we're testing panic
    // recovery behavior and the mutex will be in a consistent state.
    let result = panic::catch_unwind(AssertUnwindSafe(move || {
        let _guard = TranscribingGuard::new(state_clone).unwrap();
        panic!("Simulated panic during transcription");
    }));

    assert!(result.is_err());
    // With parking_lot::Mutex, the lock is NOT poisoned after a panic,
    // so we can safely acquire it and verify the state was reset.
    assert_eq!(*state.lock(), TranscriptionState::Idle);
}

#[test]
fn test_guard_fails_when_model_not_loaded() {
    let state = Arc::new(Mutex::new(TranscriptionState::Unloaded));
    let result = TranscribingGuard::new(state.clone());
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    assert_eq!(*state.lock(), TranscriptionState::Unloaded);
}

// ==================== Transcription Lock Tests ====================
// Mutual exclusion is critical behavior - keep concurrency tests

#[test]
fn test_transcription_lock_blocks_concurrent_access() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    let model = SharedTranscriptionModel::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let model_clone = model.clone();
        let counter_clone = counter.clone();
        let max_concurrent_clone = max_concurrent.clone();

        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                // parking_lot::Mutex::lock() returns the guard directly, no unwrap needed
                let _guard = model_clone.acquire_transcription_lock();
                let current = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
                max_concurrent_clone.fetch_max(current, Ordering::SeqCst);
                thread::sleep(Duration::from_micros(10));
                counter_clone.fetch_sub(1, Ordering::SeqCst);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(max_concurrent.load(Ordering::SeqCst), 1);
}

#[test]
fn test_transcription_lock_released_on_error_paths() {
    let model = SharedTranscriptionModel::new();

    // Error should release lock
    let _ = model.transcribe_samples(vec![], 16000, 1);

    // Lock should be acquirable again - with parking_lot, this always succeeds
    let _guard = model.acquire_transcription_lock();
    // If we get here, the lock was successfully acquired
}

// ==================== Unload/Reload Tests ====================
// These test the unload and reload functionality for system wake handling

#[test]
fn test_unload_sets_model_to_none_and_state_to_unloaded() {
    let model = SharedTranscriptionModel::new();

    // Set up a state as if model was loaded
    {
        let mut state = model.state.lock().unwrap();
        *state = TranscriptionState::Idle;
    }

    // Unload should succeed
    let result = model.unload();
    assert!(result.is_ok());

    // Model should be unloaded
    assert!(!model.is_loaded());

    // State should be Unloaded
    assert_eq!(model.state(), TranscriptionState::Unloaded);
}

#[test]
fn test_unload_releases_lock_after_completion() {
    let model = SharedTranscriptionModel::new();

    // Unload
    let _ = model.unload();

    // Lock should be acquirable again
    let guard = model.acquire_transcription_lock();
    assert!(guard.is_ok());
}

#[test]
fn test_reload_fails_with_invalid_path() {
    let model = SharedTranscriptionModel::new();

    // Set up initial state
    {
        let mut state = model.state.lock().unwrap();
        *state = TranscriptionState::Idle;
    }

    // Reload with invalid path should fail
    let result = model.reload(Path::new("/nonexistent/model/path"));
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));

    // State should be Unloaded (reload unloads first, then fails to load)
    assert_eq!(model.state(), TranscriptionState::Unloaded);
}

#[test]
fn test_reload_releases_lock_after_failure() {
    let model = SharedTranscriptionModel::new();

    // Reload with invalid path
    let _ = model.reload(Path::new("/nonexistent/model/path"));

    // Lock should be acquirable again
    let guard = model.acquire_transcription_lock();
    assert!(guard.is_ok());
}

#[test]
fn test_unload_is_thread_safe() {
    use std::thread;

    let model = SharedTranscriptionModel::new();

    // Set up initial state
    {
        let mut state = model.state.lock().unwrap();
        *state = TranscriptionState::Idle;
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let model_clone = model.clone();
        handles.push(thread::spawn(move || {
            // Unload should not panic even when called concurrently
            let _ = model_clone.unload();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Final state should be Unloaded
    assert_eq!(model.state(), TranscriptionState::Unloaded);
}
