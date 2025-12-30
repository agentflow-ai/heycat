/* v8 ignore file -- @preserve */
import { useEffect, type ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { RouterProvider } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { queryClient } from "./lib/queryClient";
import { router } from "./routes";
import { setupEventBridge } from "./lib/eventBridge";
import { useAppStore } from "./stores/appStore";
import { initializeSettings } from "./hooks/useSettings";
import { ToastProvider } from "./components/overlays";

/**
 * Component that initializes app state on mount:
 * 1. Loads settings from Tauri Store into Zustand
 * 2. Sets up the Event Bridge for backend events
 *
 * This is placed inside the QueryClientProvider so it has access to the query client.
 */
function AppInitializer({ children }: { children: ReactNode }) {
  useEffect(() => {
    let isMounted = true;
    let cleanup: (() => void) | undefined;

    const init = async () => {
      // Initialize settings from Tauri Store into Zustand
      // This happens before event bridge setup to ensure settings are available
      await initializeSettings();

      // Check if component was unmounted during async operation
      if (!isMounted) return;

      // Pre-initialize audio monitor for instant audio settings UI
      // This starts the AVAudioEngine so it's ready when user opens settings
      try {
        await invoke("init_audio_monitor");
      } catch (e) {
        console.warn("Failed to pre-initialize audio monitor:", e);
        // Non-fatal - monitor will start on-demand when settings opened
      }

      // Check again after async operation
      if (!isMounted) return;

      // Initialize event bridge with query client and store
      const store = useAppStore.getState();
      const cleanupFn = await setupEventBridge(queryClient, store);

      // Only assign cleanup if still mounted to prevent race condition
      if (isMounted) {
        cleanup = cleanupFn;
      } else {
        // Component unmounted during setup - clean up immediately
        cleanupFn();
      }
    };

    init();

    return () => {
      isMounted = false;
      cleanup?.();
    };
  }, []);

  return <>{children}</>;
}

/**
 * Root App component with provider hierarchy.
 *
 * Provider order (outermost to innermost):
 * 1. QueryClientProvider - Tanstack Query for server state caching
 * 2. ToastProvider - Toast notifications
 * 3. AppInitializer - Event Bridge setup
 * 4. RouterProvider - React Router for navigation
 */
function App() {
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

export default App;
