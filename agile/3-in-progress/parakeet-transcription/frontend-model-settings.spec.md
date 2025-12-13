---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies: ["multi-file-model-download.spec.md"]
review_round: 1
---

# Spec: Frontend model and mode settings UI

## Description

Create a new `TranscriptionSettings` React component that provides UI for:
1. Downloading TDT (batch) and EOU (streaming) models independently
2. Toggling between batch and streaming transcription modes
3. Displaying download progress for each model
4. Showing current model availability status

This component will be integrated into the existing Settings/Sidebar structure alongside the existing CommandSettings component.

## Acceptance Criteria

- [ ] New `TranscriptionSettings.tsx` component created in `src/components/Settings/`
- [ ] Component displays two model download sections: "Batch (TDT)" and "Streaming (EOU)"
- [ ] Each model section shows: download button, status indicator, progress bar during download
- [ ] Mode toggle allows switching between "Batch" and "Streaming" modes
- [ ] Mode toggle is disabled if the required model is not downloaded
- [ ] Download progress events (`model_file_download_progress`) update progress bars in real-time
- [ ] Model availability is checked on component mount via `check_model_status` command
- [ ] Selected mode is persisted (via backend command or local storage)
- [ ] Component follows existing CSS patterns from `CommandSettings.css`
- [ ] Component is accessible (proper ARIA labels, keyboard navigation)

## Test Cases

- [ ] Component renders with both model sections visible
- [ ] TDT download button triggers `download_model` with model_type="tdt"
- [ ] EOU download button triggers `download_model` with model_type="eou"
- [ ] Progress bar updates when `model_file_download_progress` event is received
- [ ] Download completion updates button to "Model Ready" state
- [ ] Mode toggle is disabled when selected model is not available
- [ ] Mode toggle calls `set_transcription_mode` command on change
- [ ] Error state displays error message below button
- [ ] Retry button appears after download error

## Dependencies

- `multi-file-model-download.spec.md` - Backend must support multi-file download with progress events and model type parameter

## Preconditions

- Backend `check_model_status` command accepts optional `model_type` parameter
- Backend `download_model` command accepts `model_type` parameter ("tdt" or "eou")
- Backend emits `model_file_download_progress` events with `{ model_type, file_name, percent }` payload
- Backend `set_transcription_mode` command exists (or mode is stored locally)

## Implementation Notes

### New Component: src/components/Settings/TranscriptionSettings.tsx

```typescript
export interface TranscriptionSettingsProps {
  className?: string;
}

export type ModelType = "tdt" | "eou";
export type TranscriptionMode = "batch" | "streaming";

interface ModelStatus {
  isAvailable: boolean;
  downloadState: DownloadState;
  progress: number; // 0-100
  error: string | null;
}
```

### Hook Modifications: src/hooks/useModelStatus.ts

Extend to support multiple models:

```typescript
export interface UseMultiModelStatusResult {
  models: Record<ModelType, ModelStatus>;
  downloadModel: (modelType: ModelType) => Promise<void>;
  refreshStatus: () => Promise<void>;
}

// New event listener for model_file_download_progress
interface ModelFileDownloadProgressPayload {
  model_type: string;
  file_name: string;
  percent: number;
  bytes_downloaded: number;
  total_bytes: number;
}
```

### Hook Modifications: src/hooks/useTranscription.ts

Add partial text state for streaming mode:

```typescript
export interface UseTranscriptionResult {
  isTranscribing: boolean;
  transcribedText: string | null;
  partialText: string | null; // NEW: For streaming mode
  error: string | null;
  durationMs: number | null;
}

// New event listener for transcription_partial
interface TranscriptionPartialPayload {
  text: string;
  is_final: boolean;
}
```

### Component Structure

```
TranscriptionSettings/
├── TranscriptionSettings.tsx
├── TranscriptionSettings.css
├── TranscriptionSettings.test.tsx
├── ModelDownloadCard.tsx (sub-component for each model)
└── ModeToggle.tsx (sub-component for mode selection)
```

### CSS Classes (following existing patterns)

```css
.transcription-settings { }
.transcription-settings__header { }
.transcription-settings__title { }
.transcription-settings__models { }
.transcription-settings__model-card { }
.transcription-settings__model-card--ready { }
.transcription-settings__model-card--downloading { }
.transcription-settings__progress-bar { }
.transcription-settings__progress-fill { }
.transcription-settings__mode-toggle { }
.transcription-settings__error { }
```

### Backend Commands to Invoke

```typescript
// Check model status
await invoke<boolean>("check_model_status", { modelType: "tdt" });

// Start download
await invoke("download_model", { modelType: "tdt" });

// Set mode
await invoke("set_transcription_mode", { mode: "batch" });

// Get current mode
await invoke<string>("get_transcription_mode");
```

### Event Listeners

```typescript
// Progress events
listen<ModelFileDownloadProgressPayload>("model_file_download_progress", (event) => {
  updateProgress(event.payload.model_type, event.payload.percent);
});

// Partial transcription (streaming)
listen<TranscriptionPartialPayload>("transcription_partial", (event) => {
  setPartialText(event.payload.text);
});
```

## Related Specs

- `multi-file-model-download.spec.md` - Provides backend support
- `wire-up-transcription.spec.md` - Consumes mode selection

## Integration Points

- Production call site: `src/App.tsx` or `src/components/Sidebar/Sidebar.tsx` (Settings section)
- Connects to: `useModelStatus` hook, `useTranscription` hook

## Integration Test

- Test location: `src/components/Settings/TranscriptionSettings.test.tsx`
- Verification: [ ] Component renders and interacts correctly with mocked invoke/listen

## Review

**Reviewed:** 2025-12-13
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| New `TranscriptionSettings.tsx` component created in `src/components/Settings/` | PASS | Component exists at `src/components/TranscriptionSettings/TranscriptionSettings.tsx:12-106`. Location differs slightly (own directory vs Settings/) but follows existing pattern of CommandSettings having its own directory. |
| Component displays two model download sections: "Batch (TDT)" and "Streaming (EOU)" | PASS | `TranscriptionSettings.tsx:72-86` renders two `ModelDownloadCard` components with titles "Batch (TDT)" and "Streaming (EOU)" |
| Each model section shows: download button, status indicator, progress bar during download | PASS | `ModelDownloadCard.tsx:74-105` shows download button, progress bar (lines 91-105) with proper state classes for status indication (lines 54-61) |
| Mode toggle allows switching between "Batch" and "Streaming" modes | PASS | `ModeToggle.tsx:21-73` implements radio group with batch/streaming options |
| Mode toggle is disabled if the required model is not downloaded | PASS | `ModeToggle.tsx:18-19` sets `isBatchDisabled` and `isStreamingDisabled` based on model availability; `ModeToggle.tsx:36,59` applies disabled attribute |
| Download progress events (`model_file_download_progress`) update progress bars in real-time | PASS | `useMultiModelStatus.ts:132-142` listens for `model_file_download_progress` and updates progress state |
| Model availability is checked on component mount via `check_model_status` command | PASS | `useMultiModelStatus.ts:129` calls `refreshStatus()` which invokes `check_model_status` for both models (lines 74-77) |
| Selected mode is persisted (via backend command or local storage) | PASS | `TranscriptionSettings.tsx:43` calls `invoke("set_transcription_mode", { mode })` on change; mode loaded from backend on mount (lines 25-26) |
| Component follows existing CSS patterns from `CommandSettings.css` | PASS | `TranscriptionSettings.css` follows same patterns: BEM naming, consistent spacing (16px padding), similar color values (#3b82f6, #1f2937, etc.), dark mode support via media query |
| Component is accessible (proper ARIA labels, keyboard navigation) | PASS | `TranscriptionSettings.tsx:62-64` has region role with aria-label; `ModelDownloadCard.tsx:78-79` has aria-label and aria-busy; `ModeToggle.tsx:24-25` has radiogroup role; progress bar has proper ARIA attributes (lines 94-98) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Component renders with both model sections visible | PASS | `TranscriptionSettings.test.tsx:54-63` |
| TDT download button triggers `download_model` with model_type="tdt" | PASS | `TranscriptionSettings.test.tsx:84-97` |
| EOU download button triggers `download_model` with model_type="eou" | PASS | `TranscriptionSettings.test.tsx:99-112` |
| Progress bar updates when `model_file_download_progress` event is received | PASS | `TranscriptionSettings.test.tsx:114-140` |
| Download completion updates button to "Model Ready" state | PASS | `TranscriptionSettings.test.tsx:142-163` |
| Mode toggle is disabled when selected model is not available | PASS | `TranscriptionSettings.test.tsx:225-247` |
| Mode toggle calls `set_transcription_mode` command on change | PASS | `TranscriptionSettings.test.tsx:269-294` |
| Error state displays error message below button | PASS | `TranscriptionSettings.test.tsx:165-192` |
| Retry button appears after download error | PASS | `TranscriptionSettings.test.tsx:194-221` |

### Code Quality

**Strengths:**
- Clean separation of concerns: main component (`TranscriptionSettings.tsx`), sub-components (`ModelDownloadCard.tsx`, `ModeToggle.tsx`), and custom hook (`useMultiModelStatus.ts`)
- Comprehensive test coverage for both the component and the hook with proper mocking of Tauri APIs
- Consistent use of TypeScript types with proper exports for reusability
- Excellent accessibility implementation with ARIA labels, roles, and keyboard support
- CSS follows BEM naming convention and includes dark mode support matching existing patterns
- Proper cleanup of event listeners in hooks (unlisten functions)
- Good error handling with user-friendly error messages and retry functionality
- `useTranscription.ts` properly updated with `partialText` state and `transcription_partial` event listener (lines 31, 43, 84-93)

**Concerns:**
- None identified

### Verdict

**APPROVED** - The implementation fully satisfies all acceptance criteria. The component structure, accessibility features, test coverage, and CSS patterns are well-aligned with existing codebase conventions. The `useTranscription` hook has been properly extended with `partialText` support for streaming mode. All specified test cases have corresponding tests that verify the expected behavior.
