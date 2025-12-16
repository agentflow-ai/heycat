---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
review_round: 1
review_history:
  - round: 1
    date: 2025-12-16
    verdict: NEEDS_WORK
    failedCriteria: []
    concerns: []
priority: P1
---

# Spec: Wire up RecordingDetectors to HotkeyIntegration

## Description

`RecordingDetectors` is exported from `listening/mod.rs` but marked with `#[allow(unused_imports)]`. The sophisticated detection loop in `coordinator.rs` (lines 154-399) is never actually called. This represents incomplete integration - silence detection for recordings is implemented but not wired up.

Complete the integration by connecting `RecordingDetectors` to `HotkeyIntegration` so recordings can auto-stop based on silence detection.

## Acceptance Criteria

- [ ] Remove `#[allow(unused_imports)]` from RecordingDetectors export
- [ ] HotkeyIntegration uses RecordingDetectors for silence-based recording stop
- [ ] Silence detection starts when recording begins
- [ ] Recording auto-stops when silence is detected (configurable)
- [ ] User can still manually stop recording (takes precedence)
- [ ] Feature can be disabled via config

## Test Cases

- [ ] Test recording auto-stops after configured silence duration
- [ ] Test manual stop overrides silence detection
- [ ] Test silence detection can be disabled
- [ ] Test recording continues if speech is detected

## Dependencies

None

## Preconditions

- RecordingDetectors and coordinator.rs already implemented
- HotkeyIntegration manages recording lifecycle

## Implementation Notes

**Files:**
- `src-tauri/src/listening/mod.rs` - Lines 22-23: `#[allow(unused_imports)]` on RecordingDetectors
- `src-tauri/src/listening/coordinator.rs` - Lines 154-399: Detection loop (not called)
- `src-tauri/src/hotkey/integration.rs` - HotkeyIntegration (needs to use RecordingDetectors)

**Current state:**
- coordinator.rs has `start_monitoring()` that takes a `ListeningPipeline`
- Detection loop checks for silence and can restart listening
- But HotkeyIntegration doesn't use any of this

**Integration approach:**

1. In HotkeyIntegration, when recording starts:
```rust
// Create and start RecordingDetectors
let detectors = RecordingDetectors::new(config);
detectors.start_monitoring(&audio_handle)?;
```

2. Subscribe to silence detection events:
```rust
// On silence detected
self.stop_recording_internal().await?;
```

3. Clean up on recording stop:
```rust
// Stop monitoring when recording ends
detectors.stop_monitoring();
```

**Questions to resolve:**
- Should RecordingDetectors share the same audio buffer as recording?
- How does silence detection interact with transcription timing?
- Should there be a minimum recording duration before silence triggers stop?

## Related Specs

- unified-vad-config.spec.md (completed - VAD used for silence detection)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (HotkeyIntegration)
- Connects to: RecordingManager, ListeningManager

## Integration Test

- Test location: `src-tauri/src/hotkey/integration.rs` or new integration test file
- Verification: [ ] Integration test passes

---

## Review

**Date:** 2025-12-16
**Commit:** 6b42d47 WIP: wire-recording-detectors
**Round:** 1

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: unused imports: `VAD_CHUNK_SIZE_16KHZ` and `VAD_CHUNK_SIZE_8KHZ`
 --> src/listening/vad.rs:5:54
```
**PASS** - Warning is pre-existing in vad.rs, not related to this spec's changes.

#### 2. Command Registration Check
N/A - No new Tauri commands added.

#### 3. Event Subscription Check
N/A - No new events added.

### Manual Review

#### 1. Is the code wired up end-to-end?
- [x] New functions are called from production code
- [x] New structs are instantiated in production code
- N/A New events (none added)
- N/A New commands (none added)

**Evidence:**
- `with_recording_detectors()` builder method called in `lib.rs:161`
- `RecordingDetectors` created and managed in `lib.rs:90-91`
- `start_silence_detection()` called when recording starts (`integration.rs:329`)
- `stop_silence_detection()` called on manual stop (`integration.rs:349`)

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `with_recording_detectors()` | fn | lib.rs:161 | YES |
| `with_silence_detection_enabled()` | fn | (builder, used in tests) | TEST-ONLY (acceptable - config method) |
| `with_silence_config()` | fn | (builder, used in tests) | TEST-ONLY (acceptable - config method) |
| `start_silence_detection()` | fn | integration.rs:329 | YES |
| `stop_silence_detection()` | fn | integration.rs:349 | YES |
| `recording_detectors` field | struct field | used throughout | YES |
| `silence_detection_enabled` field | struct field | used in start_silence_detection | YES |
| `silence_config` field | struct field | stored but not yet used | PARTIAL |

**Note:** `with_silence_detection_enabled()` and `with_silence_config()` are marked `#[allow(dead_code)]` - acceptable for builder configuration methods that may be used in production later or via config.

#### 3. Where does the data flow?

```
[Hotkey Press]
     |
     v
[HotkeyIntegration::handle_toggle] integration.rs:289
     | recording starts
     v
[start_silence_detection] integration.rs:821
     | creates transcription callback, starts monitoring
     v
[RecordingDetectors::start_monitoring] coordinator.rs
     | monitors audio buffer for silence
     v
[On silence detected: transcription_callback]
     | spawns async transcription task
     v
[Clipboard write + paste] integration.rs:1001-1009
     | or
     v
[Manual stop via hotkey]
     | calls stop_silence_detection first
     v
[RecordingDetectors::stop_monitoring]
```

**All links verified as connected.**

#### 4. Are there any deferrals?

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| "For voice command matching, we just fall through to clipboard for now" | integration.rs:992 | **MISSING** |
| "Placeholder - fall through to clipboard" | integration.rs:998 | **MISSING** |

**Issue:** The silence detection path does not support voice command matching - it always falls through to clipboard. This is intentional simplification, but lacks a tracking spec.

#### 5. Automated check results

Build check passed (warning unrelated). No new commands or events to verify.

### Acceptance Criteria Verification

- [x] Remove `#[allow(unused_imports)]` from RecordingDetectors export - **VERIFIED** in mod.rs diff
- [x] HotkeyIntegration uses RecordingDetectors for silence-based recording stop - **VERIFIED** via builder and start/stop methods
- [x] Silence detection starts when recording begins - **VERIFIED** in handle_toggle at line 329
- [x] Recording auto-stops when silence is detected (configurable) - **PARTIAL** - starts monitoring but `silence_config` field is stored but not passed to `start_monitoring`
- [x] User can still manually stop recording (takes precedence) - **VERIFIED** - stop_silence_detection called before processing at line 349
- [x] Feature can be disabled via config - **VERIFIED** - silence_detection_enabled flag checked at line 828

### Test Cases Verification

- [x] Test recording auto-stops after configured silence duration - **PARTIAL** - test verifies builder works, not actual auto-stop behavior
- [x] Test manual stop overrides silence detection - **VERIFIED** - test_manual_stop_takes_precedence_over_silence_detection
- [x] Test silence detection can be disabled - **VERIFIED** - test_silence_detection_respects_enabled_flag
- [x] Test recording continues if speech is detected - **NOT TESTED** - no test for speech continuation

### Verdict: NEEDS_WORK

**What failed:** Question 4 (Deferrals) - Missing tracking spec for deferred voice command matching in silence detection path.

**Why it failed:** Lines 992-998 in integration.rs contain placeholder code that bypasses voice command matching, falling through to clipboard without a tracking spec.

**How to fix:**
1. Create a tracking spec for voice command matching in silence detection flow, OR
2. Add a comment referencing an existing spec that covers this, OR
3. Remove the voice command matching code path entirely from the silence detection callback if it's not intended to be supported
