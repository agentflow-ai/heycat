---
status: pending
created: 2025-12-22
completed: null
dependencies: ["add-resampler-tests"]
---

# Spec: Verify fix with 10+ consecutive recordings and sample ratio validation

## Description

Perform manual verification that the progressive audio speedup bug is fixed by making 10+ consecutive recordings and validating that audio plays back at consistent speed. Also verify sample ratio logging shows consistent values across all recordings.

## Acceptance Criteria

- [ ] 10+ consecutive recordings made successfully
- [ ] All recordings play back at the same speed (no progressive speedup)
- [ ] Sample ratio logs show consistent values (within 0.1% across recordings)
- [ ] Audio quality is consistent (no metallic/robotic artifacts)
- [ ] Transcription quality remains consistent

## Test Cases

- [ ] Recording 1 plays at normal speed
- [ ] Recording 5 plays at same speed as Recording 1
- [ ] Recording 10 plays at same speed as Recording 1
- [ ] Sample ratio in logs: Recording 1 vs Recording 10 within 0.1%
- [ ] Transcribe all recordings - quality consistent

## Dependencies

- `add-resampler-tests.spec.md` - regression tests should pass first

## Preconditions

- All previous specs completed
- Unit tests passing
- Device uses resampling (48kHz → 16kHz)

## Implementation Notes

**Verification Steps:**

1. **Build and launch app**
   ```bash
   cargo tauri dev
   ```

2. **Make 10+ consecutive recordings using hotkey**
   - Keep recordings short (5-10 seconds each)
   - Say similar phrases for comparison

3. **Check sample ratio logs**
   - Look for "Sample ratio:" in logs
   - All recordings should show same ratio (e.g., 0.333 for 48→16kHz)
   - Variance should be < 0.1%

4. **Play back recordings**
   - Compare speed of first vs last recording
   - Listen for metallic/robotic artifacts
   - Should be indistinguishable

5. **Transcription quality check**
   - Compare transcription accuracy across recordings
   - Should be consistent

**Pass Criteria:**
- No audible difference in playback speed between recordings
- Sample ratio variance < 0.1% across all recordings
- No metallic/robotic audio artifacts
- Transcription accuracy consistent

## Related Specs

All previous specs in this bug issue

## Integration Points

- Production call site: Full application integration
- Connects to: Audio capture, resampling, WAV encoding, transcription

## Integration Test

Manual verification test procedure as described above

- Test location: Manual testing with running application
- Verification: [ ] Integration test passes
