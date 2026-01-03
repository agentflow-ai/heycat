---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Frontend Patterns

```
                              STATE LAYERS
───────────────────────────────────────────────────────────────────────
                    ┌─────────────────────────────────┐
                    │         COMPONENTS              │
                    │   (consume state via hooks)     │
                    └─────────────────────────────────┘
                           │           │           │
              ┌────────────┘           │           └────────────┐
              ▼                        ▼                        ▼
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Zustand Store   │     │  Tanstack Query  │     │  Tauri Store     │
│  (client state)  │     │  (server state)  │     │  (persistence)   │
│                  │     │                  │     │                  │
│  • overlayMode   │     │  • recordings    │     │  • settings.json │
│  • settingsCache │     │  • modelStatus   │     │                  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
        │                        │                        │
        │                        ▼                        │
        │               ┌──────────────────┐              │
        └──────────────▶│     BACKEND      │◀─────────────┘
         dual-write     │   invoke<T>()    │    persist
                        └──────────────────┘
```

## Tauri Integration

**All `invoke()` calls must be in custom hooks**, never in components directly.

```typescript
// hooks/useAudioDevices.ts
export function useAudioDevices(): UseAudioDevicesResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.listAudioDevices,
    queryFn: () => invoke<AudioInputDevice[]>("list_audio_devices"),
  });

  return {
    devices: data ?? [],
    isLoading,
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
  };
}
```

- Always use typed `invoke<T>()` with explicit return type
- Query keys: use centralized `queryKeys` from `src/lib/queryKeys.ts`
- Mutations: no `onSuccess` invalidation (Event Bridge handles it)

## State Layers

| Layer | Tool | Purpose |
|-------|------|---------|
| Client | Zustand (`appStore.ts`) | UI state, settings cache |
| Server | Tanstack Query | Backend data (recordings, model status) |
| Persistent | Tauri Store | Settings the backend needs |

- **Never put server data in Zustand** - use Query hooks
- Use Zustand selectors: `useOverlayMode()` not `useAppStore()`
- Settings require dual-write: Zustand (fast UI) + Tauri Store (persistence)

```typescript
async function updateSettingInBothStores<K extends keyof AppSettings>(
  key: K, nestedKey: keyof AppSettings[K], value: AppSettings[K][keyof AppSettings[K]]
): Promise<void> {
  useAppStore.getState().updateSetting(key, { [nestedKey]: value });
  const store = await load(settingsFile);
  await store.set(`${key}.${String(nestedKey)}`, value);
  await store.save();
}
```

## Errors

- Convert unknown errors: `e instanceof Error ? e.message : String(e)`
- Return type: `Error | null` (not `string | null`)
- Never swallow errors silently - at minimum log them

```typescript
// In catch blocks
setError(e instanceof Error ? e.message : "Failed to play audio");

// In hook returns
error: error instanceof Error ? error : error ? new Error(String(error)) : null
```
