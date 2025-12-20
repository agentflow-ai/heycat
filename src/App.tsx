/* v8 ignore file -- @preserve */
import { useEffect, type ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { RouterProvider } from "react-router-dom";
import { queryClient } from "./lib/queryClient";
import { router } from "./routes";
import { setupEventBridge } from "./lib/eventBridge";
import { useAppStore } from "./stores/appStore";
import { ToastProvider } from "./components/overlays";

/**
 * Component that initializes the Event Bridge on mount and cleans up on unmount.
 * This is placed inside the QueryClientProvider so it has access to the query client.
 */
function AppInitializer({ children }: { children: ReactNode }) {
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    // Initialize event bridge with query client and store
    const store = useAppStore.getState();
    setupEventBridge(queryClient, store).then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    return () => {
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
