import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { X } from "lucide-react";
import { Button } from "../../components/ui";

export interface ShortcutEditorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  shortcutName: string;
  currentShortcut: string;
  onSave: (displayShortcut: string, backendShortcut: string) => void;
}

// Backend captured key event structure
interface CapturedKeyEvent {
  key_code: number;
  key_name: string;
  fn_key: boolean;
  command: boolean;
  control: boolean;
  alt: boolean;
  shift: boolean;
  pressed: boolean;
}

// Convert backend key event to display string (e.g., "fn⌘⇧R")
function formatBackendKeyForDisplay(event: CapturedKeyEvent): string {
  const parts: string[] = [];

  // Add modifiers in standard order (fn first since it's special)
  if (event.fn_key) parts.push("fn");
  if (event.command) parts.push("⌘");
  if (event.control) parts.push("⌃");
  if (event.alt) parts.push("⌥");
  if (event.shift) parts.push("⇧");

  // Add the main key (excluding modifier keys themselves)
  const isModifierKey = ["Command", "Control", "Alt", "Shift", "fn"].includes(event.key_name);
  if (!isModifierKey) {
    // Special key mappings for display
    const keyMap: Record<string, string> = {
      Up: "↑",
      Down: "↓",
      Left: "←",
      Right: "→",
      Enter: "↵",
      Backspace: "⌫",
      Delete: "⌦",
      Escape: "Esc",
      Tab: "⇥",
      Space: "Space",
    };
    parts.push(keyMap[event.key_name] || event.key_name);
  }

  return parts.join("");
}

// Convert backend key event to backend format (e.g., "Function+Command+Shift+R")
function formatBackendKeyForBackend(event: CapturedKeyEvent): string {
  const parts: string[] = [];

  // Add modifiers in standard order
  // Note: Tauri's global-shortcut uses "Function" for fn key
  if (event.fn_key) parts.push("Function");
  if (event.command) parts.push("Command");
  if (event.control) parts.push("Control");
  if (event.alt) parts.push("Alt");
  if (event.shift) parts.push("Shift");

  // Add the main key (excluding modifier keys themselves)
  const isModifierKey = ["Command", "Control", "Alt", "Shift", "fn"].includes(event.key_name);
  if (!isModifierKey) {
    parts.push(event.key_name);
  }

  return parts.join("+");
}

export function ShortcutEditor({
  open,
  onOpenChange,
  shortcutName,
  currentShortcut,
  onSave,
}: ShortcutEditorProps) {
  const [recording, setRecording] = useState(false);
  const [recordedShortcut, setRecordedShortcut] = useState<{
    display: string;
    backend: string;
  } | null>(null);
  const [shortcutSuspended, setShortcutSuspended] = useState(false);
  const [permissionError, setPermissionError] = useState<string | null>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Suspend global shortcut when entering recording mode
  const suspendShortcut = useCallback(async () => {
    if (shortcutSuspended) return;
    try {
      await invoke("suspend_recording_shortcut");
      setShortcutSuspended(true);
    } catch (error) {
      console.error("Failed to suspend recording shortcut:", error);
    }
  }, [shortcutSuspended]);

  // Resume global shortcut when exiting recording mode
  const resumeShortcut = useCallback(async () => {
    if (!shortcutSuspended) return;
    try {
      await invoke("resume_recording_shortcut");
      setShortcutSuspended(false);
    } catch (error) {
      console.error("Failed to resume recording shortcut:", error);
    }
  }, [shortcutSuspended]);

  // Start backend keyboard capture
  const startCapture = useCallback(async () => {
    try {
      // Start the backend keyboard capture
      await invoke("start_shortcut_recording");
      console.log("[ShortcutEditor] Backend keyboard capture started");
      setPermissionError(null);
    } catch (error) {
      console.error("Failed to start keyboard capture:", error);
      const errorMessage = String(error);

      // Check if this is a permission error
      if (errorMessage.includes("Input Monitoring permission")) {
        setPermissionError(errorMessage);
      }

      setRecording(false);
    }
  }, []);

  // Stop backend keyboard capture
  const stopCapture = useCallback(async () => {
    try {
      await invoke("stop_shortcut_recording");
      console.log("[ShortcutEditor] Backend keyboard capture stopped");
    } catch (error) {
      console.error("Failed to stop keyboard capture:", error);
    }
  }, []);

  // Open System Preferences to Input Monitoring
  const openInputMonitoringPreferences = useCallback(async () => {
    try {
      await invoke("open_input_monitoring_preferences");
    } catch (error) {
      console.error("Failed to open preferences:", error);
    }
  }, []);

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      setRecording(false);
      setRecordedShortcut(null);
      setShortcutSuspended(false);
      setPermissionError(null);
    }
  }, [open]);

  // Handle backend key events when recording
  useEffect(() => {
    if (!recording) {
      // Clean up listener when not recording
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      return;
    }

    // Start backend capture and listen for events
    let isMounted = true;

    const setupCapture = async () => {
      // Start backend capture
      await startCapture();

      // Listen for captured key events
      const unlisten = await listen<CapturedKeyEvent>("shortcut_key_captured", (event) => {
        if (!isMounted) return;

        const keyEvent = event.payload;
        console.log("[ShortcutEditor] Key captured from backend:", keyEvent);

        // Only process key press events (not releases)
        if (!keyEvent.pressed) return;

        // Check if this is just a modifier key by itself
        const isModifierKey = ["Command", "Control", "Alt", "Shift", "fn"].includes(keyEvent.key_name);

        if (!isModifierKey) {
          // Non-modifier key pressed - record the shortcut
          const display = formatBackendKeyForDisplay(keyEvent);
          const backend = formatBackendKeyForBackend(keyEvent);
          console.log("[ShortcutEditor] Recording shortcut - display:", display, "backend:", backend);
          setRecordedShortcut({ display, backend });
          setRecording(false);
          // Stop capture after recording
          stopCapture();
        }
      });

      unlistenRef.current = unlisten;
    };

    setupCapture();

    return () => {
      isMounted = false;
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      stopCapture();
    };
  }, [recording, startCapture, stopCapture]);

  // Handle click outside to close
  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (dialogRef.current && !dialogRef.current.contains(e.target as Node)) {
        stopCapture();
        resumeShortcut();
        onOpenChange(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, onOpenChange, resumeShortcut, stopCapture]);

  // Handle Escape to close (when not recording) - still use JS event for this
  useEffect(() => {
    if (!open) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !recording) {
        stopCapture();
        resumeShortcut();
        onOpenChange(false);
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [open, recording, onOpenChange, resumeShortcut, stopCapture]);

  if (!open) return null;

  const displayShortcut = recordedShortcut?.display ?? currentShortcut;
  const hasChanges = recordedShortcut !== null && recordedShortcut.display !== currentShortcut;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50" aria-hidden="true" />

      {/* Modal */}
      <div
        ref={dialogRef}
        role="dialog"
        aria-labelledby="shortcut-editor-title"
        aria-modal="true"
        className="
          relative
          bg-surface
          rounded-lg
          shadow-xl
          border border-border
          w-full max-w-md
          p-6
          animate-in fade-in zoom-in-95
        "
      >
        {/* Close button */}
        <button
          type="button"
          onClick={() => {
            stopCapture();
            resumeShortcut();
            onOpenChange(false);
          }}
          className="
            absolute top-4 right-4
            p-1 rounded
            text-text-secondary hover:text-text-primary
            transition-colors
          "
          aria-label="Close"
        >
          <X className="h-5 w-5" />
        </button>

        {/* Header */}
        <h2
          id="shortcut-editor-title"
          className="text-lg font-semibold text-text-primary mb-1"
        >
          Change Keyboard Shortcut
        </h2>
        <p className="text-sm text-text-secondary mb-6">
          Set a new shortcut for "{shortcutName}"
        </p>

        {/* Shortcut Display */}
        <div className="mb-6">
          <div
            className={`
              flex items-center justify-center
              h-20
              bg-surface
              border-2 rounded-lg
              transition-colors
              ${
                recording
                  ? "border-heycat-teal border-dashed animate-pulse"
                  : permissionError
                    ? "border-red-500"
                    : "border-border"
              }
            `}
          >
            {recording ? (
              <span className="text-text-secondary">Press your shortcut...</span>
            ) : (
              <kbd className="px-4 py-2 text-2xl font-mono bg-surface-elevated text-text-primary border border-border rounded-lg">
                {displayShortcut}
              </kbd>
            )}
          </div>
        </div>

        {/* Permission Error */}
        {permissionError && (
          <div className="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
            <p className="text-sm text-red-400 mb-3">
              Input Monitoring permission is required to capture the fn key.
            </p>
            <Button
              variant="secondary"
              onClick={openInputMonitoringPreferences}
              className="w-full"
            >
              Open System Settings
            </Button>
            <p className="text-xs text-text-secondary mt-2 text-center">
              After granting permission, restart the app and try again.
            </p>
          </div>
        )}

        {/* Actions */}
        <div className="flex items-center justify-between">
          <Button
            variant="secondary"
            onClick={async () => {
              // Suspend global shortcut before entering recording mode
              await suspendShortcut();
              setRecording(true);
              setRecordedShortcut(null);
            }}
            disabled={recording}
          >
            {recording ? "Recording..." : "Record New Shortcut"}
          </Button>

          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => {
              stopCapture();
              resumeShortcut();
              onOpenChange(false);
            }}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                if (recordedShortcut) {
                  onSave(recordedShortcut.display, recordedShortcut.backend);
                }
              }}
              disabled={!hasChanges}
            >
              Save
            </Button>
          </div>
        </div>

        {/* Help text */}
        <p className="mt-4 text-xs text-text-secondary text-center">
          Press any key or combination (e.g., fn, F1, ⌘R, fn⌘R)
        </p>
      </div>
    </div>
  );
}
