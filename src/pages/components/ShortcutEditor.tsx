import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import { Button } from "../../components/ui";
import { useSettings } from "../../hooks/useSettings";
import { useShortcutRecorder } from "../../hooks/useShortcutRecorder";

export interface ShortcutEditorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  shortcutName: string;
  currentShortcut: string;
  onSave: (displayShortcut: string, backendShortcut: string) => void;
}

export function ShortcutEditor({
  open,
  onOpenChange,
  shortcutName,
  currentShortcut,
  onSave,
}: ShortcutEditorProps) {
  const { settings } = useSettings();
  const [shortcutSuspended, setShortcutSuspended] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);

  // Use the shortcut recorder hook
  const {
    isRecording: recording,
    recordedShortcut,
    permissionError,
    startRecording,
    stopRecording,
    clearRecordedShortcut,
    openAccessibilityPreferences,
  } = useShortcutRecorder({
    distinguishLeftRight: settings.shortcuts?.distinguishLeftRight ?? false,
  });

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

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      clearRecordedShortcut();
      setShortcutSuspended(false);
    }
  }, [open, clearRecordedShortcut]);

  // Handle click outside to close
  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (dialogRef.current && !dialogRef.current.contains(e.target as Node)) {
        stopRecording();
        resumeShortcut();
        onOpenChange(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, onOpenChange, resumeShortcut, stopRecording]);

  // Handle Escape to close (when not recording) - still use JS event for this
  useEffect(() => {
    if (!open) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !recording) {
        stopRecording();
        resumeShortcut();
        onOpenChange(false);
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [open, recording, onOpenChange, resumeShortcut, stopRecording]);

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
            stopRecording();
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
              Accessibility permission is required to capture fn key and media keys.
            </p>
            <Button
              variant="secondary"
              onClick={openAccessibilityPreferences}
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
              await startRecording();
            }}
            disabled={recording}
          >
            {recording ? "Recording..." : "Record New Shortcut"}
          </Button>

          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => {
              stopRecording();
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
