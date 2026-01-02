---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Frontend Error Handling Pattern

## Error Conversion Pattern

Always convert unknown errors to `Error` type using `instanceof Error`:

```typescript
// In hooks and callbacks
setError(e instanceof Error ? e.message : "Failed to play audio");

// For returning Error | null
error: error instanceof Error ? error : error ? new Error(String(error)) : null
```

This handles:
- Standard `Error` objects
- String errors from Tauri commands
- Unknown error types

## Hook Return Types

Use `Error | null` for typed error returns:

```typescript
export interface UseRecordingStateResult {
  isRecording: boolean;
  isProcessing: boolean;
  isLoading: boolean;
  error: Error | null;  // Not string | null
}

export function useRecordingState(): UseRecordingStateResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.getRecordingState,
    queryFn: () => invoke<RecordingStateResponse>("get_recording_state"),
  });

  return {
    isRecording: data?.state === "Recording",
    isProcessing: data?.state === "Processing",
    isLoading,
    // Convert query error to Error | null
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
  };
}
```

## Try-Catch in Async Operations

Use try-catch with proper error conversion:

```typescript
const play = useCallback(async (filePath: string) => {
  try {
    await audioRef.current.play();
    setIsPlaying(true);
  } catch (e) {
    // Convert unknown to string message
    setError(e instanceof Error ? e.message : "Failed to play audio");
    setIsPlaying(false);
  }
}, []);
```

## Combining Multiple Error Sources

When hooks use multiple mutations, combine their errors:

```typescript
// Combine errors: query error, or mutation errors
const combinedError = error?.message
  ?? (startMutation.error instanceof Error ? startMutation.error.message : null)
  ?? (stopMutation.error instanceof Error ? stopMutation.error.message : null)
  ?? null;

return {
  // ...
  error: combinedError,
};
```

## Mutation Error Handlers

Use `onError` callback in mutations for error-specific logic:

```typescript
useMutation({
  mutationFn: downloadModels,
  onError: (error) => {
    setDownloadState("error");
    setDownloadError(error instanceof Error ? error.message : String(error));
  },
});
```

## Anti-Patterns

### Unhandled promise rejections

```typescript
// BAD: Unhandled rejection
async function handleClick() {
  await invoke("risky_command");  // Throws if command fails
}

// GOOD: Wrapped in try-catch
async function handleClick() {
  try {
    await invoke("risky_command");
  } catch (e) {
    setError(e instanceof Error ? e.message : "Operation failed");
  }
}
```

### Assuming error is Error type

```typescript
// BAD: Error might be string or other type
catch (e) {
  setError(e.message);  // Crashes if e is not Error
}

// GOOD: Type check first
catch (e) {
  setError(e instanceof Error ? e.message : String(e));
}
```

### Returning string | null instead of Error | null

```typescript
// BAD: Loses stack trace, less type-safe
export interface MyResult {
  error: string | null;
}

// GOOD: Preserves error information
export interface MyResult {
  error: Error | null;
}
```

### Silent error swallowing

```typescript
// BAD: Error is silently ignored
catch (e) {
  // Do nothing
}

// GOOD: At minimum, log the error
catch (e) {
  console.error("[heycat] Operation failed:", e);
}
```
