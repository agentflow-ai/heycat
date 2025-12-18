import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import { Button } from "../../components/ui";

export interface ShortcutEditorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  shortcutName: string;
  currentShortcut: string;
  onSave: (displayShortcut: string, backendShortcut: string) => void;
}

// Convert key event to display string (e.g., "⌘⇧R")
function formatKeyForDisplay(e: KeyboardEvent): string {
  const parts: string[] = [];

  // Add modifiers in standard order
  if (e.metaKey) parts.push("⌘");
  if (e.ctrlKey) parts.push("⌃");
  if (e.altKey) parts.push("⌥");
  if (e.shiftKey) parts.push("⇧");

  // Add the main key (excluding modifier keys themselves)
  const key = e.key;
  if (!["Meta", "Control", "Alt", "Shift"].includes(key)) {
    // Special key mappings
    const keyMap: Record<string, string> = {
      ArrowUp: "↑",
      ArrowDown: "↓",
      ArrowLeft: "←",
      ArrowRight: "→",
      Enter: "↵",
      Backspace: "⌫",
      Delete: "⌦",
      Escape: "Esc",
      Tab: "⇥",
      " ": "Space",
    };
    parts.push(keyMap[key] || key.toUpperCase());
  }

  return parts.join("");
}

// Convert key event to backend format (e.g., "CmdOrControl+Shift+R")
function formatKeyForBackend(e: KeyboardEvent): string {
  const parts: string[] = [];

  // Add modifiers in standard order
  if (e.metaKey || e.ctrlKey) parts.push("CmdOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  // Add the main key (excluding modifier keys themselves)
  const key = e.key;
  if (!["Meta", "Control", "Alt", "Shift"].includes(key)) {
    // Normalize key names for the backend
    const keyMap: Record<string, string> = {
      ArrowUp: "Up",
      ArrowDown: "Down",
      ArrowLeft: "Left",
      ArrowRight: "Right",
      " ": "Space",
    };
    parts.push(keyMap[key] || key.toUpperCase());
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
  const dialogRef = useRef<HTMLDivElement>(null);

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

  // Reset state when modal opens/closes
  useEffect(() => {
    if (open) {
      setRecording(false);
      setRecordedShortcut(null);
    } else {
      // Ensure shortcut is resumed when modal closes
      if (shortcutSuspended) {
        resumeShortcut();
      }
    }
  }, [open, shortcutSuspended, resumeShortcut]);

  // Handle keyboard events when recording
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!recording) return;

      e.preventDefault();
      e.stopPropagation();

      // Record any key (with or without modifiers)
      // Skip if only a modifier key is pressed by itself
      const isModifierKey = ["Meta", "Control", "Alt", "Shift"].includes(e.key);

      if (!isModifierKey) {
        setRecordedShortcut({
          display: formatKeyForDisplay(e),
          backend: formatKeyForBackend(e),
        });
        setRecording(false);
        // Resume the global shortcut after successful recording
        resumeShortcut();
      }
    },
    [recording, resumeShortcut]
  );

  // Add/remove keyboard listener
  useEffect(() => {
    if (recording) {
      window.addEventListener("keydown", handleKeyDown, true);
      return () => window.removeEventListener("keydown", handleKeyDown, true);
    }
  }, [recording, handleKeyDown]);

  // Handle click outside to close
  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (dialogRef.current && !dialogRef.current.contains(e.target as Node)) {
        onOpenChange(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, onOpenChange]);

  // Handle Escape to close (when not recording)
  useEffect(() => {
    if (!open) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !recording) {
        onOpenChange(false);
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [open, recording, onOpenChange]);

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
          onClick={() => onOpenChange(false)}
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
            <Button variant="ghost" onClick={() => onOpenChange(false)}>
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
          Press any key or combination (e.g., F1, ⌘R, ⌘⇧R)
        </p>
      </div>
    </div>
  );
}
