---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
priority: P1
---

# Spec: Consolidate WakeWordDetector 4 Mutexes into single state struct

## Description

`WakeWordDetector` uses 4 separate Mutex instances for its internal state (lines 188-195 in detector.rs). This creates deadlock risk due to unspecified lock ordering, complicates the code, and adds unnecessary contention.

Consolidate all mutable state into a single `DetectorState` struct wrapped in one Mutex.

## Acceptance Criteria

- [ ] Create `DetectorState` struct containing all mutable detector state
- [ ] Replace 4 Mutexes with single `Mutex<DetectorState>`
- [ ] All methods acquire single lock instead of multiple locks
- [ ] Document that lock is coarse-grained for simplicity
- [ ] All existing tests pass
- [ ] No deadlock risk (single lock = no ordering issues)

## Test Cases

- [ ] Test concurrent access from multiple threads
- [ ] Test no performance regression (single lock should be faster)
- [ ] Test all detector operations work correctly
- [ ] Stress test with rapid analysis cycles

## Dependencies

None

## Preconditions

- Current WakeWordDetector with 4 separate Mutexes

## Implementation Notes

**File:** `src-tauri/src/listening/detector.rs`

**Current structure (lines 188-195):**
```rust
pub struct WakeWordDetector {
    config: WakeWordDetectorConfig,
    buffer: Mutex<CircularBuffer>,
    last_analysis_sample_count: Mutex<u64>,
    recent_fingerprints: Mutex<VecDeque<AudioFingerprint>>,
    vad: Mutex<Option<VoiceActivityDetector>>,
    shared_model: Option<Arc<SharedTranscriptionModel>>,
}
```

**Proposed structure:**
```rust
/// Internal mutable state for WakeWordDetector.
/// All fields are protected by a single lock for simplicity and deadlock prevention.
struct DetectorState {
    buffer: CircularBuffer,
    last_analysis_sample_count: u64,
    recent_fingerprints: VecDeque<AudioFingerprint>,
    vad: Option<VoiceActivityDetector>,
}

pub struct WakeWordDetector {
    config: WakeWordDetectorConfig,
    state: Mutex<DetectorState>,  // Single lock!
    shared_model: Option<Arc<SharedTranscriptionModel>>,
}
```

**Methods to update:**
- `push_samples()` - Acquires state lock once
- `analyze()` / `analyze_and_emit()` - Acquires state lock for full operation
- `check_vad()` - Already internal, just use state.vad
- `init_vad()` - Acquires state lock to set vad
- `clear_buffer()` - Acquires state lock

**Benefits:**
1. No deadlock possible (single lock)
2. Simpler code (one lock acquisition per operation)
3. Better cache locality (state is contiguous)
4. Easier to reason about thread safety

**Tradeoff:**
- Coarser granularity means longer lock hold times
- But analysis interval is 150ms, so contention is minimal
- Simplicity > micro-optimization here

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/listening/pipeline.rs`
- Connects to: ListeningPipeline (analysis thread)

## Integration Test

- Test location: `src-tauri/src/listening/detector.rs` (test module)
- Verification: [ ] Integration test passes
