---
status: pending
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Add type parameter to stop_recording invoke

## Description

Add explicit type parameter to the `invoke("stop_recording")` call in `src/hooks/useRecording.ts`. While the return value isn't used (state comes from events), adding the type parameter improves code documentation and TypeScript type safety. The `stop_recording` Tauri command returns `RecordingMetadata`.

## Acceptance Criteria

- [ ] `invoke("stop_recording")` has explicit type parameter `invoke<RecordingMetadata>("stop_recording")`
- [ ] TypeScript compilation succeeds without errors
- [ ] Recording stop functionality works as before

## Test Cases

- [ ] TypeScript type checking passes
- [ ] Recording can be started and stopped via UI
- [ ] Recording stopped event is received correctly

## Dependencies

None

## Preconditions

- `RecordingMetadata` interface is already defined in the file

## Implementation Notes

Update `src/hooks/useRecording.ts:65`:

```typescript
// Before
await invoke("stop_recording");

// After
await invoke<RecordingMetadata>("stop_recording");
```

Note: The return value still isn't used (state sync happens via events), but the type parameter documents the expected return type and enables TypeScript to catch type mismatches.

## Related Specs

None - isolated TypeScript improvement

## Integration Points

- Production call site: `src/hooks/useRecording.ts:65`
- Connects to: `src-tauri/src/commands/mod.rs:224` (Rust command)

## Integration Test

- Test location: N/A (type annotation change, no runtime behavior change)
- Verification: [x] N/A
