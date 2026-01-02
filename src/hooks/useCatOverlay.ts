import { useEffect, useRef } from "react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { primaryMonitor, LogicalPosition } from "@tauri-apps/api/window";
import { useRecording } from "./useRecording";
import { useSettings } from "./useSettings";
import { useAppStore } from "../stores/appStore";

const OVERLAY_LABEL = "cat-overlay";
const OVERLAY_SIZE = 120;

/** Overlay visual mode based on app state */
export type OverlayMode = "hidden" | "recording";

function getOverlayUrl(): string {
  if (import.meta.env.DEV) {
    // Use dynamic port for worktree support (default 1420, worktrees use 1421-1429)
    const port = import.meta.env.VITE_DEV_PORT || "1420";
    return `http://localhost:${port}/overlay.html`;
  }
  return "/overlay.html";
}

async function calculateOverlayPosition(): Promise<{ x: number; y: number } | null> {
  const monitor = await primaryMonitor();
  if (!monitor) return null;

  const monitorPosition = monitor.position;
  const monitorSize = monitor.size;
  const scale = monitor.scaleFactor;

  const logicalWidth = monitorSize.width / scale;
  const logicalHeight = monitorSize.height / scale;
  const logicalX = monitorPosition.x / scale;
  const logicalY = monitorPosition.y / scale;

  return {
    x: Math.round(logicalX + (logicalWidth - OVERLAY_SIZE) / 2),
    y: Math.round(logicalY + logicalHeight - OVERLAY_SIZE - 50),
  };
}

export function useCatOverlay() {
  const { settings } = useSettings();
  const { isRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });
  const initializedRef = useRef(false);

  // Determine the overlay mode based on state
  const overlayMode: OverlayMode = isRecording ? "recording" : "hidden";

  // Sync overlay mode to Zustand store for global access
  const setOverlayMode = useAppStore((s) => s.setOverlayMode);
  useEffect(() => {
    setOverlayMode(overlayMode === "hidden" ? null : overlayMode);
  }, [overlayMode, setOverlayMode]);

  // Initialize overlay window once (hidden) on mount
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    if (initializedRef.current) return;
    initializedRef.current = true;

    const initOverlay = async () => {
      const existing = await WebviewWindow.getByLabel(OVERLAY_LABEL);
      if (existing) return;

      const position = await calculateOverlayPosition();
      if (!position) return;

      const overlayWindow = new WebviewWindow(OVERLAY_LABEL, {
        url: getOverlayUrl(),
        width: OVERLAY_SIZE,
        height: OVERLAY_SIZE,
        x: position.x,
        y: position.y,
        transparent: true,
        decorations: false,
        alwaysOnTop: true,
        resizable: false,
        skipTaskbar: true,
        focus: false,
        visible: false,
      });

      overlayWindow.once("tauri://created", async () => {
        await overlayWindow.setIgnoreCursorEvents(true);
      });
    };

    initOverlay();
    /* v8 ignore stop */
  }, []);

  // Show/hide and update mode based on state
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    const updateOverlay = async () => {
      const window = await WebviewWindow.getByLabel(OVERLAY_LABEL);
      if (!window) return;

      const shouldShow = overlayMode !== "hidden";

      if (shouldShow) {
        // Recalculate position in case monitor setup changed
        const position = await calculateOverlayPosition();
        if (position) {
          await window.setPosition(new LogicalPosition(position.x, position.y));
        }
        // Emit mode to overlay window for visual distinction
        await window.emit("overlay_mode", { mode: overlayMode });
        await window.show();
      } else {
        await window.hide();
      }
    };

    updateOverlay();
    /* v8 ignore stop */
  }, [overlayMode]);

  return { isRecording, overlayMode };
}
