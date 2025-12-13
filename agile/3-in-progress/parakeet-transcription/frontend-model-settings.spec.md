---
status: pending
created: 2025-12-13
completed: null
dependencies: ["multi-file-model-download.spec.md"]
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
