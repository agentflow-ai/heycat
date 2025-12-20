---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["query-infrastructure", "event-bridge"]
---

# Spec: Migrate remaining data hooks

## Description

Convert the remaining data-fetching hooks to Tanstack Query: `useAudioDevices`, `useMultiModelStatus`, and `useTranscription`. These are read-heavy hooks that benefit from Query's caching and automatic refetching. The `useAudioLevelMonitor` hook is intentionally NOT migrated (20fps updates are too frequent for Query).

## Acceptance Criteria

### useAudioDevices
- [ ] `useAudioDevices.ts` refactored to use `useQuery`
- [ ] Query key: `['tauri', 'list_audio_devices']`
- [ ] Replace `setInterval` polling with `refetchInterval` option
- [ ] `refetchOnWindowFocus: true` for device hot-plug detection
- [ ] Returns: `{ devices, isLoading, error, refetch }`

### useMultiModelStatus
- [ ] `useMultiModelStatus.ts` refactored to use `useQuery`
- [ ] Query key: `['tauri', 'check_parakeet_model_status', modelType]`
- [ ] Download progress events update via Event Bridge invalidation
- [ ] `useDownloadModel()` mutation for triggering downloads
- [ ] Returns: `{ status, isDownloading, progress, downloadModel }`

### useTranscription
- [ ] `useTranscription.ts` refactored for event-driven updates
- [ ] Transcription state managed via Event Bridge → Zustand (UI state)
- [ ] Or via Query if there's a `get_transcription_status` command
- [ ] Progress and completion events handled appropriately

### Preserved Hooks (no migration)
- [ ] `useAudioLevelMonitor.ts` - NOT migrated (20fps too fast for Query)
- [ ] `useAppStatus.ts` - May derive from Zustand or compose other hooks

## Test Cases

- [ ] `useAudioDevices()` returns device list from cache
- [ ] Device list refreshes on window focus
- [ ] `useMultiModelStatus('tdt')` returns model status
- [ ] Model download progress updates in real-time
- [ ] Transcription events update UI state correctly
- [ ] Audio level monitor still works at 20fps (not broken)

## Dependencies

- `query-infrastructure` - provides queryClient, queryKeys
- `event-bridge` - handles model download events

## Preconditions

- QueryClientProvider wrapping app
- Event Bridge initialized

## Implementation Notes

```typescript
// src/hooks/useAudioDevices.ts
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { queryKeys } from '../lib/queryKeys';

interface AudioDevice {
  name: string;
  isDefault: boolean;
}

export function useAudioDevices() {
  return useQuery({
    queryKey: queryKeys.tauri.listAudioDevices,
    queryFn: () => invoke<AudioDevice[]>('list_audio_devices'),
    refetchInterval: 5000, // Poll every 5s for hot-plug
    refetchOnWindowFocus: true,
  });
}
```

```typescript
// src/hooks/useMultiModelStatus.ts
import { useQuery, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { queryKeys } from '../lib/queryKeys';

interface ModelStatus {
  isAvailable: boolean;
  isDownloading: boolean;
  downloadProgress: number;
}

export function useModelStatus(modelType: string) {
  return useQuery({
    queryKey: queryKeys.tauri.checkModelStatus(modelType),
    queryFn: () => invoke<ModelStatus>('check_parakeet_model_status', { modelType }),
  });
}

export function useDownloadModel() {
  return useMutation({
    mutationFn: (modelType: string) => invoke('download_model', { modelType }),
    // Event Bridge invalidates on model_download_completed
  });
}
```

```typescript
// src/hooks/useTranscription.ts
// This hook may need special handling - transcription is event-driven
// Consider if it should use Zustand for transcription state
// or if there's a query-able endpoint

import { useAppStore } from '../stores/appStore';

export function useTranscription() {
  // If transcription state is UI state, use Zustand
  const transcriptionState = useAppStore((s) => s.transcriptionState);

  return {
    isTranscribing: transcriptionState?.isProcessing ?? false,
    result: transcriptionState?.result ?? null,
    error: transcriptionState?.error ?? null,
  };
}
```

**Hooks NOT being migrated:**
- `useAudioLevelMonitor` - 20fps real-time updates, keep as local useState + listen()
- `useCatOverlay` - Complex WebviewWindow API, keep existing pattern
- `useDisambiguation` - Event-driven command flow, may need Zustand
- `useAutoStartListening` - One-time init effect, doesn't need Query

## Related Specs

- `query-infrastructure` - provides query infrastructure
- `event-bridge` - invalidates model status on download events
- `zustand-store` - may hold transcription UI state

## Integration Points

- Production call site: Settings (audio devices), Dashboard (model status)
- Connects to: queryClient, Event Bridge, Tauri backend

## Integration Test

- Test location: `src/hooks/__tests__/dataHooks.test.ts`
- Verification: [ ] Audio devices, model status queries work correctly
