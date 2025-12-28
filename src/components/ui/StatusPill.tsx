import { type HTMLAttributes, forwardRef, useState, useEffect } from "react";
import { Loader2 } from "lucide-react";

export type StatusPillStatus = "idle" | "recording" | "processing";

export interface StatusPillProps extends HTMLAttributes<HTMLDivElement> {
  /** Current status to display */
  status: StatusPillStatus;
  /** Recording duration in seconds (only shown for recording status) */
  recordingDuration?: number;
  /** Custom label override */
  label?: string;
}

const statusConfig: Record<
  StatusPillStatus,
  { bgClass: string; textClass: string; label: string; animation: string }
> = {
  idle: {
    bgClass: "bg-neutral-400",
    textClass: "text-white",
    label: "Ready",
    animation: "",
  },
  recording: {
    bgClass: "bg-recording",
    textClass: "text-white",
    label: "Recording",
    animation: "status-pill-pulse",
  },
  processing: {
    bgClass: "bg-processing",
    textClass: "text-white",
    label: "Processing...",
    animation: "",
  },
};

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export const StatusPill = forwardRef<HTMLDivElement, StatusPillProps>(
  (
    { status, recordingDuration, label, className = "", ...props },
    ref
  ) => {
    const config = statusConfig[status];
    const displayLabel = label ?? config.label;

    const showDuration = status === "recording" && recordingDuration !== undefined;
    const showSpinner = status === "processing";

    return (
      <div
        ref={ref}
        className={`
          inline-flex items-center gap-2
          px-3 py-1.5
          rounded-full
          ${config.bgClass}
          ${config.animation}
          transition-all duration-[var(--duration-normal)]
          ${className}
        `}
        role="status"
        aria-label={`Status: ${displayLabel}${showDuration ? `, duration ${formatDuration(recordingDuration!)}` : ""}`}
        aria-live="polite"
        {...props}
      >
        {showSpinner && (
          <Loader2
            className="w-4 h-4 text-white animate-spin"
            aria-hidden="true"
          />
        )}
        <span className={`text-sm font-medium ${config.textClass}`}>
          {displayLabel}
        </span>
        {showDuration && (
          <span
            className={`text-sm font-mono ${config.textClass} opacity-90`}
            aria-hidden="true"
          >
            {formatDuration(recordingDuration!)}
          </span>
        )}
      </div>
    );
  }
);

StatusPill.displayName = "StatusPill";

/**
 * StatusPill with automatic duration tracking for recording state.
 * Starts timer when status changes to "recording" and stops when it changes away.
 */
export interface AutoTimerStatusPillProps
  extends Omit<StatusPillProps, "recordingDuration"> {
  /** Initial duration offset in seconds (for resuming) */
  initialDuration?: number;
}

export const AutoTimerStatusPill = forwardRef<
  HTMLDivElement,
  AutoTimerStatusPillProps
>(({ status, initialDuration = 0, ...props }, ref) => {
  const [duration, setDuration] = useState(initialDuration);

  useEffect(() => {
    if (status !== "recording") {
      setDuration(0);
      return;
    }

    setDuration(initialDuration);
    const interval = setInterval(() => {
      setDuration((prev) => prev + 1);
    }, 1000);

    return () => clearInterval(interval);
  }, [status, initialDuration]);

  return (
    <StatusPill
      ref={ref}
      status={status}
      recordingDuration={status === "recording" ? duration : undefined}
      {...props}
    />
  );
});

AutoTimerStatusPill.displayName = "AutoTimerStatusPill";
