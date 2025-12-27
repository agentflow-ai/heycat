import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ActiveWindowChangedPayload } from "../types/windowContext";

/**
 * Hook for tracking the currently active window.
 *
 * Listens to `active_window_changed` events from the backend and provides
 * the current active window info along with any matched context.
 *
 * Note: This hook requires the WindowMonitor to be running on the backend
 * (to be implemented in window-monitor.spec.md).
 */
export function useActiveWindow() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindowChangedPayload | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      unlisten = await listen<ActiveWindowChangedPayload>(
        "active_window_changed",
        (event) => {
          setActiveWindow(event.payload);
        }
      );
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, []);

  return {
    activeWindow,
    matchedContextId: activeWindow?.matchedContextId ?? null,
    matchedContextName: activeWindow?.matchedContextName ?? null,
  };
}
