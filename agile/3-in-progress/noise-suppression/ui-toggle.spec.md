---
status: in-review
created: 2025-12-23
completed: null
dependencies: [pipeline-integration]
review_round: 1
---

# Spec: Add settings UI toggle to enable/disable noise suppression

## Description

Add a toggle switch in the Audio Settings panel to let users enable or disable noise suppression. Currently noise suppression is always-on; this gives users control when they prefer raw audio (e.g., quiet environments, external processing, or debugging).

The toggle will:
1. Add `noiseSuppression: boolean` to `AudioSettings` type (default: `true`)
2. Display a Toggle component in the Audio Input section of AudioSettings
3. Persist the setting via `useSettings` hook (Zustand + Tauri Store)
4. Backend reads setting and bypasses denoiser when disabled

## Acceptance Criteria

- [ ] Toggle appears in Audio Settings under "Audio Input" section
- [ ] Toggle defaults to ON (enabled) for new installations
- [ ] Toggle state persists across app restarts
- [ ] Backend respects the setting (denoiser bypassed when OFF)
- [ ] Toast notification confirms setting change

## Test Cases

- [ ] `AudioSettings.test.tsx`: Toggle renders and is checked by default
- [ ] `AudioSettings.test.tsx`: Clicking toggle updates settings state
- [ ] `useSettings.test.ts`: `updateNoiseSuppression(false)` persists to store
- [ ] Backend: `cargo test` - audio capture respects `noise_suppression` setting

## Dependencies

- `pipeline-integration` (denoiser must be integrated before we can toggle it)

## Preconditions

- Noise suppression pipeline is working (denoiser integrated into cpal_backend)
- Toggle component exists in `src/components/ui/Toggle.tsx`

## Implementation Notes

### Frontend Changes

1. **`src/types/audio.ts`** - Add field to `AudioSettings`:
   ```typescript
   noiseSuppression: boolean;
   ```

2. **`src/hooks/useSettings.ts`**:
   - Add `updateNoiseSuppression` method
   - Load `audio.noiseSuppression` in `initializeSettings()`

3. **`src/pages/components/AudioSettings.tsx`**:
   - Import `Toggle` component
   - Add toggle row below Audio Level Meter
   - Call `updateNoiseSuppression` on change

### Backend Changes

4. **`src-tauri/src/audio/cpal_backend.rs`**:
   - Read `audio.noiseSuppression` from Tauri Store on capture start
   - Skip denoiser processing when setting is `false`

## Related Specs

- [pipeline-integration.spec.md](./pipeline-integration.spec.md) - Denoiser integration

## Integration Points

- Production call site: `src/pages/components/AudioSettings.tsx` (Toggle component)
- Connects to: `useSettings` hook, Tauri Store, `cpal_backend.rs`

## Integration Test

- Test location: `src/pages/components/AudioSettings.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Pre-Review Gates (Automated)

#### 1. Build Warning Check
```
warning: method `get` is never used
   --> src/dictionary/store.rs:227:12
```
**PASS** - Warning is from dictionary module, not related to this spec.

#### 2. Command Registration Check
Not applicable - no new commands added.

#### 3. Event Subscription Check
Not applicable - no new events added.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Toggle appears in Audio Settings under "Audio Input" section | PASS | `src/pages/components/AudioSettings.tsx:177-183` - LabeledToggle rendered inside Audio Input section |
| Toggle defaults to ON (enabled) for new installations | PASS | `src/types/audio.ts:24` - `noiseSuppression: true` in DEFAULT_AUDIO_SETTINGS |
| Toggle state persists across app restarts | PASS | `src/hooks/useSettings.ts:178` - persists via `updateSettingInBothStores("audio", "noiseSuppression", enabled)` |
| Backend respects the setting (denoiser bypassed when OFF) | PASS | `src-tauri/src/commands/mod.rs:197-211` - reads setting and conditionally sets `denoiser_for_recording` to `None` |
| Toast notification confirms setting change | PASS | `src/pages/components/AudioSettings.tsx:73-82` - `handleNoiseSuppressionChange` calls `toast()` with success message |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Toggle renders and is checked by default | MISSING | No `AudioSettings.test.tsx` found |
| Clicking toggle updates settings state | MISSING | No `AudioSettings.test.tsx` found |
| `updateNoiseSuppression(false)` persists to store | PASS | `src/hooks/useSettings.test.ts:161-179` |
| Backend: audio capture respects `noise_suppression` setting | PASS | `cargo test` passes (422 tests) |

### Code Quality

**Strengths:**
- Clean separation of concerns: type definition, hook logic, UI component, backend logic
- Uses existing `LabeledToggle` component for consistent UI
- Default-safe: noise suppression enabled by default, only disabled if explicitly set to `false`
- Toast feedback provides clear user confirmation
- Backend reads directly from Tauri Store, avoiding frontend-backend sync issues

**Concerns:**
- Missing `AudioSettings.test.tsx` file - spec lists test cases in this file but file does not exist
- The acceptance criteria checkboxes are unchecked, indicating verification was not documented

### Data Flow Verification

```
[UI Action] User toggles noise suppression
     |
     v
[Handler] src/pages/components/AudioSettings.tsx:73 handleNoiseSuppressionChange()
     | await updateNoiseSuppression(checked)
     v
[Hook] src/hooks/useSettings.ts:177-178 updateNoiseSuppression()
     | await updateSettingInBothStores("audio", "noiseSuppression", enabled)
     v
[Store] Zustand updated immediately + Tauri Store persisted
     |
     v
[UI Re-render] Toggle reflects new state from settings.audio.noiseSuppression
     |
     v
[Toast] Success notification displayed

--- On Next Recording ---

[Command] src-tauri/src/commands/mod.rs:197-211 start_recording()
     | store.get("audio.noiseSuppression")
     v
[Logic] If false, denoiser_for_recording = None, bypassing denoiser
```

**PASS** - Data flow is complete with no broken links.

### Verdict

**NEEDS_WORK** - Missing AudioSettings component tests

1. **What failed**: Test Coverage Audit - spec lists `AudioSettings.test.tsx` tests that do not exist
2. **Why it failed**: The file `src/pages/components/AudioSettings.test.tsx` does not exist, so the following test cases are missing:
   - "Toggle renders and is checked by default"
   - "Clicking toggle updates settings state"
3. **How to fix**: Create `src/pages/components/AudioSettings.test.tsx` with tests for:
   - Toggle renders in the Audio Input section and is checked by default (tests default value from settings)
   - Clicking toggle calls `updateNoiseSuppression` with the toggled value
