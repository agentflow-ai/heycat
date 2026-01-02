export interface ShortcutDisplayProps {
  /** The shortcut to display */
  shortcut: string;
  /** Whether currently recording a new shortcut */
  isRecording: boolean;
  /** Whether there's a permission error */
  hasPermissionError: boolean;
}

/**
 * Component for displaying the current or recorded keyboard shortcut.
 */
export function ShortcutDisplay({
  shortcut,
  isRecording,
  hasPermissionError,
}: ShortcutDisplayProps) {
  return (
    <div className="mb-6">
      <div
        className={`
          flex items-center justify-center
          h-20
          bg-surface
          border-2 rounded-lg
          transition-colors
          ${
            isRecording
              ? "border-heycat-teal border-dashed animate-pulse"
              : hasPermissionError
                ? "border-red-500"
                : "border-border"
          }
        `}
      >
        {isRecording ? (
          <span className="text-text-secondary">Press your shortcut...</span>
        ) : (
          <kbd className="px-4 py-2 text-2xl font-mono bg-surface-elevated text-text-primary border border-border rounded-lg">
            {shortcut}
          </kbd>
        )}
      </div>
    </div>
  );
}
