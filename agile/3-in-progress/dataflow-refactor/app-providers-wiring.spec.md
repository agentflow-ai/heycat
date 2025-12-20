---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["query-infrastructure", "zustand-store", "event-bridge", "router-setup"]
---

# Spec: Wire all providers in App.tsx

## Description

Update App.tsx to integrate all the new infrastructure: wrap the app with QueryClientProvider and RouterProvider, initialize the Event Bridge on mount, and remove the old useState-based navigation pattern. This is the "wiring spec" that connects all foundational pieces.

## Acceptance Criteria

- [ ] App.tsx updated with provider hierarchy:
  1. `QueryClientProvider` (outermost, wraps everything)
  2. `ToastProvider` (existing, preserved)
  3. `RouterProvider` (replaces manual page switching)
- [ ] Event Bridge initialized in useEffect on app mount
- [ ] Event Bridge cleanup called on app unmount
- [ ] Settings loaded into Zustand store on app init
- [ ] Old `useState` navigation pattern removed:
  - Remove `const [navItem, setNavItem] = useState('dashboard')`
  - Remove conditional page rendering `{navItem === 'dashboard' && ...}`
  - Remove `onNavigate` prop passing
- [ ] React Query DevTools included in development mode
- [ ] App renders without errors after changes
- [ ] Existing functionality preserved (recording, listening, settings)

## Test Cases

- [ ] App mounts without console errors
- [ ] Navigation between pages works via URL
- [ ] QueryClientProvider context available in components
- [ ] Event Bridge listeners are active (test with mock event)
- [ ] Settings are loaded into Zustand on mount
- [ ] Cleanup runs on unmount (no memory leaks)

## Dependencies

- `query-infrastructure` - provides queryClient
- `zustand-store` - provides useAppStore
- `event-bridge` - provides setupEventBridge
- `router-setup` - provides router

## Preconditions

- All dependency specs completed
- Existing App.tsx structure understood

## Implementation Notes

```typescript
// src/App.tsx
import { useEffect } from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { RouterProvider } from 'react-router-dom';
import { queryClient } from './lib/queryClient';
import { router } from './routes';
import { setupEventBridge } from './lib/eventBridge';
import { useAppStore } from './stores/appStore';
import { ToastProvider } from './components/overlays/toast/ToastProvider';

function AppInitializer({ children }: { children: React.ReactNode }) {
  const store = useAppStore.getState();

  useEffect(() => {
    // Initialize event bridge
    let cleanup: (() => void) | undefined;

    setupEventBridge(queryClient, store).then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    // Load settings into Zustand (from Tauri Store)
    // This will be handled by settings-zustand-hooks spec

    return () => {
      cleanup?.();
    };
  }, []);

  return <>{children}</>;
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <AppInitializer>
          <RouterProvider router={router} />
        </AppInitializer>
      </ToastProvider>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
```

**Key changes from current App.tsx:**
- Remove: `useState('dashboard')` navigation state
- Remove: Conditional rendering based on navItem
- Remove: `onNavigate` prop to child components
- Add: Provider wrapper hierarchy
- Add: Event Bridge initialization

## Related Specs

- All foundation specs are dependencies
- All hook migration specs depend on this being complete

## Integration Points

- Production call site: This IS the production entry point
- Connects to: queryClient, router, eventBridge, appStore, all pages

## Integration Test

- Test location: `src/__tests__/App.test.tsx`
- Verification: [ ] App renders and navigation works end-to-end
