---
status: pending
created: 2025-12-12
completed: null
dependencies:
  - auto-transcribe-on-stop
---

# Spec: Transcription UI Components

## Description

Create frontend UI components to display transcription state, show success/error notifications, and block new recordings during transcription. This provides visual feedback for the transcription workflow.

## Acceptance Criteria

- [ ] `TranscriptionIndicator` component shows "Transcribing..." during transcription
- [ ] Loading indicator visible while isTranscribing is true
- [ ] New recordings blocked while transcribing (disable UI, show message)
- [ ] Success notification shown on transcription completion (brief, auto-dismiss)
- [ ] Error notification shown on transcription failure (persistent until dismissed)
- [ ] Proper accessibility: aria-live="polite", aria-busy, role="status"
- [ ] Components integrated into App.tsx

## Test Cases

- [ ] TranscriptionIndicator shows loading state when isTranscribing=true
- [ ] TranscriptionIndicator hidden when isTranscribing=false
- [ ] Success notification displays transcribed text preview
- [ ] Error notification displays error message
- [ ] Recording button disabled during transcription
- [ ] Accessibility attributes present on status elements

## Dependencies

- auto-transcribe-on-stop (useTranscription hook, events)

## Preconditions

- useTranscription hook provides isTranscribing, lastTranscription, error states
- Tauri events are properly emitted from backend

## Implementation Notes

- Follow RecordingIndicator pattern for TranscriptionIndicator
- Use inline notifications (no external library per frontend patterns)
- Success notification: show first ~50 chars of text + "Copied to clipboard"
- Error notification: show error message with dismiss button
- Consider toast-style notifications that auto-dismiss

```typescript
// useTranscription hook interface
interface UseTranscriptionResult {
  isTranscribing: boolean;
  lastTranscription: string | null;
  error: string | null;
}

// TranscriptionIndicator props
interface TranscriptionIndicatorProps {
  isTranscribing: boolean;
}
```

## Related Specs

- auto-transcribe-on-stop.spec.md (provides useTranscription hook)
- model-download.spec.md (ModelDownloadButton in same UI area)

## Integration Points

- Production call site: `src/App.tsx`
- Connects to: useTranscription hook, RecordingIndicator

## Integration Test

- Test location: `src/components/TranscriptionIndicator.test.tsx`
- Verification: [ ] Integration test passes
