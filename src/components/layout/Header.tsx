import { Cat, Settings, HelpCircle, Command } from "lucide-react";
import { StatusIndicator, type StatusIndicatorVariant } from "../ui";

export interface HeaderProps {
  /** Current status for the status pill */
  status?: "idle" | "listening" | "recording" | "processing";
  /** Status label override */
  statusLabel?: string;
  /** Callback when command palette trigger is clicked */
  onCommandPaletteOpen?: () => void;
  /** Callback when settings is clicked */
  onSettingsClick?: () => void;
  /** Callback when help is clicked */
  onHelpClick?: () => void;
}

const statusToVariant: Record<string, StatusIndicatorVariant> = {
  idle: "idle",
  listening: "listening",
  recording: "recording",
  processing: "processing",
};

const defaultStatusLabels: Record<string, string> = {
  idle: "Idle",
  listening: "Listening...",
  recording: "Recording",
  processing: "Processing",
};

export function Header({
  status = "idle",
  statusLabel,
  onCommandPaletteOpen,
  onSettingsClick,
  onHelpClick,
}: HeaderProps) {
  const displayLabel = statusLabel ?? defaultStatusLabels[status];

  return (
    <header
      className="h-12 flex items-center justify-between px-4 border-b border-border bg-surface shrink-0"
      role="banner"
    >
      {/* Left: Logo */}
      <div className="flex items-center gap-2">
        <Cat
          className="w-6 h-6 text-heycat-orange"
          aria-hidden="true"
        />
        <span className="text-lg font-semibold text-text-primary">
          HeyCat
        </span>
      </div>

      {/* Center: Status Pill */}
      <div className="absolute left-1/2 -translate-x-1/2">
        <StatusIndicator variant={statusToVariant[status]} label={displayLabel} />
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-1">
        {/* Command Palette Trigger */}
        <button
          type="button"
          onClick={onCommandPaletteOpen}
          className="
            flex items-center gap-1.5 px-2 py-1
            text-sm text-text-secondary
            bg-neutral-100 hover:bg-neutral-200
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Open command palette (Command K)"
        >
          <Command className="w-3.5 h-3.5" aria-hidden="true" />
          <span>K</span>
        </button>

        {/* Settings */}
        <button
          type="button"
          onClick={onSettingsClick}
          className="
            p-2
            text-text-secondary hover:text-text-primary
            hover:bg-neutral-100
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Settings"
        >
          <Settings className="w-5 h-5" aria-hidden="true" />
        </button>

        {/* Help */}
        <button
          type="button"
          onClick={onHelpClick}
          className="
            p-2
            text-text-secondary hover:text-text-primary
            hover:bg-neutral-100
            rounded-[var(--radius-sm)]
            transition-colors duration-[var(--duration-fast)]
          "
          aria-label="Help"
        >
          <HelpCircle className="w-5 h-5" aria-hidden="true" />
        </button>
      </div>
    </header>
  );
}
