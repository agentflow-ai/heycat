---
status: completed
created: 2025-12-16
completed: 2025-12-16
dependencies: []
review_round: 1
priority: P0
---

# Spec: Add semaphore to prevent concurrent batch+streaming transcription

## Description

The `SharedTranscriptionModel` has a critical race condition: `transcribe_file()` (batch mode) uses the RAII state guard, but `transcribe_samples()` (streaming mode for wake word) bypasses the state machine entirely. Both methods can run simultaneously on the same model, causing latency spikes and unpredictable behavior.

Add a semaphore or exclusive access mechanism to ensure batch and streaming transcription cannot execute concurrently.

## Acceptance Criteria

- [ ] Add `Semaphore` or `RwLock` to `SharedTranscriptionModel` to prevent concurrent transcription
- [ ] `transcribe_file()` acquires exclusive access before transcribing
- [ ] `transcribe_samples()` acquires exclusive access before transcribing
- [ ] If one mode is active, the other blocks or returns an error
- [ ] Document the mutual exclusion in code comments
- [ ] No deadlock introduced by new locking

## Test Cases

- [ ] Test concurrent calls to `transcribe_file()` + `transcribe_samples()` are serialized
- [ ] Test that two `transcribe_file()` calls don't interleave
- [ ] Test that semaphore is released on error paths
- [ ] Stress test with rapid alternating batch/streaming calls

## Dependencies

None

## Preconditions

- Existing `SharedTranscriptionModel` with separate batch and streaming paths

## Implementation Notes

**File:** `src-tauri/src/parakeet/shared.rs`

**Current state:**
- Lines 207-248: `transcribe_file()` uses `TranscribingGuard` (state machine)
- Lines 258-290: `transcribe_samples()` bypasses state (lines 270-272 comment explains why)

**Options:**
1. Use `tokio::sync::Semaphore` with 1 permit
2. Use `parking_lot::RwLock` for reader (streaming) / writer (batch) semantics
3. Use `std::sync::Mutex` wrapper around transcription operations

**Recommended approach:**
```rust
pub struct SharedTranscriptionModel {
    model: Arc<Mutex<Option<ParakeetModelWrapper>>>,
    state: Arc<Mutex<TranscriptionState>>,
    transcription_lock: Arc<Semaphore>,  // NEW: 1 permit
}

pub async fn transcribe_samples(&self, ...) -> Result<...> {
    let _permit = self.transcription_lock.acquire().await?;
    // ... existing logic
}

pub async fn transcribe_file(&self, ...) -> Result<...> {
    let _permit = self.transcription_lock.acquire().await?;
    // ... existing logic with guard
}
```

## Related Specs

- shared-transcription-model.spec.md (completed - original SharedTranscriptionModel)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs:385` (streaming)
- Production call site: `src-tauri/src/hotkey/integration.rs` (batch)
- Connects to: ListeningPipeline, HotkeyIntegration

## Integration Test

- Test location: `src-tauri/src/parakeet/shared.rs` (unit tests section)
- Verification: [ ] Integration test passes

---

## Review

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: unused imports: `VAD_CHUNK_SIZE_16KHZ` and `VAD_CHUNK_SIZE_8KHZ`
```
**PASS** - Warning is unrelated to this spec (in vad.rs, not shared.rs)

#### 2. Command Registration Check
N/A - Spec does not add new commands

#### 3. Event Subscription Check
N/A - Spec does not add new events

### Manual Review

#### 1. Is the code wired up end-to-end?

- [x] New functions are called from production code (not just tests)
- [x] New structs are instantiated in production code (not just tests)
- N/A New events - no events added
- N/A New commands - no commands added

The `transcription_lock` field (line 129) and `acquire_transcription_lock()` method (line 153) are called from both:
- `transcribe_file()` at line 244
- `transcribe_samples()` at line 313

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `transcription_lock` field | struct field | SharedTranscriptionModel:129 | YES (via transcribe_file/transcribe_samples) |
| `acquire_transcription_lock()` | fn | shared.rs:244, shared.rs:313 | YES |

Production paths:
- `transcribe_samples()` called from `src-tauri/src/listening/detector.rs:393` (wake word detection)
- `transcribe_file()` called from `src-tauri/src/commands/logic.rs:448` via `TranscriptionService` trait (hotkey recording)

#### 3. Where does the data flow?

```
[Wake Word Detection]
     |
     v
[WakeWordDetector] src-tauri/src/listening/detector.rs:393
     | shared_model.transcribe_samples()
     v
[SharedTranscriptionModel] src-tauri/src/parakeet/shared.rs:313
     | _transcription_permit = acquire_transcription_lock()
     v
[Mutex Guard] - ensures exclusivity
     |
     v
[ParakeetTDT::transcribe_samples()]
```

```
[Hotkey Recording Transcription]
     |
     v
[Command] src-tauri/src/commands/mod.rs:258
     | transcribe_file_impl()
     v
[Logic] src-tauri/src/commands/logic.rs:448
     | shared_model.transcribe()
     v
[SharedTranscriptionModel] src-tauri/src/parakeet/shared.rs:244
     | _transcription_permit = acquire_transcription_lock()
     v
[Mutex Guard] - ensures exclusivity
     |
     v
[ParakeetTDT::transcribe_file()]
```

Both paths acquire the same `transcription_lock` mutex, ensuring mutual exclusion.

#### 4. Are there any deferrals?

No deferrals found in implementation. `grep` for TODO/FIXME/XXX/HACK returned no matches in shared.rs.

#### 5. Automated check results

```
$ cargo test shared --no-fail-fast
running 28 tests
test parakeet::shared::tests::test_transcription_lock_is_acquired ... ok
test parakeet::shared::tests::test_transcription_lock_blocks_concurrent_access ... ok
test parakeet::shared::tests::test_transcription_lock_released_on_error_paths ... ok
test parakeet::shared::tests::test_transcription_lock_released_on_model_not_loaded ... ok
test parakeet::shared::tests::test_two_transcribe_file_calls_are_serialized ... ok
test parakeet::shared::tests::test_stress_alternating_batch_streaming_calls ... ok
[...all 28 tests pass...]
test result: ok. 28 passed; 0 failed; 0 ignored
```

### Acceptance Criteria Verification

- [x] Add `Semaphore` or `RwLock` to `SharedTranscriptionModel` to prevent concurrent transcription
  - `transcription_lock: Arc<Mutex<()>>` added at line 129
- [x] `transcribe_file()` acquires exclusive access before transcribing
  - Line 244: `let _transcription_permit = self.acquire_transcription_lock()?;`
- [x] `transcribe_samples()` acquires exclusive access before transcribing
  - Line 313: `let _transcription_permit = self.acquire_transcription_lock()?;`
- [x] If one mode is active, the other blocks or returns an error
  - Mutex blocks until lock is released
- [x] Document the mutual exclusion in code comments
  - Lines 105-109, 127-128, 230-235, 291-295
- [x] No deadlock introduced by new locking
  - Lock is acquired first, before any other locks; released via RAII

### Test Cases Verification

- [x] Test concurrent calls to `transcribe_file()` + `transcribe_samples()` are serialized
  - `test_stress_alternating_batch_streaming_calls` (10 threads, 200 total operations, max concurrent = 1)
- [x] Test that two `transcribe_file()` calls don't interleave
  - `test_two_transcribe_file_calls_are_serialized`
- [x] Test that semaphore is released on error paths
  - `test_transcription_lock_released_on_error_paths`, `test_transcription_lock_released_on_model_not_loaded`
- [x] Stress test with rapid alternating batch/streaming calls
  - `test_stress_alternating_batch_streaming_calls`

### Verdict

**APPROVED**

- [x] All automated checks pass (no warnings in shared.rs, all registrations verified)
- [x] All new code is reachable from production (not test-only)
- [x] Data flow is complete with no broken links
- [x] All deferrals reference tracking specs (none present)
