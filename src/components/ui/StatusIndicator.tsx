import { type HTMLAttributes, forwardRef } from "react";

export type StatusIndicatorVariant = "recording" | "processing" | "idle";

export interface StatusIndicatorProps extends HTMLAttributes<HTMLDivElement> {
  variant: StatusIndicatorVariant;
  /**
   * Optional label to show next to the indicator
   */
  label?: string;
  /**
   * Size of the indicator dot
   */
  size?: "sm" | "md" | "lg";
}

const sizeClasses: Record<"sm" | "md" | "lg", { dot: string; text: string }> = {
  sm: { dot: "h-2 w-2", text: "text-xs" },
  md: { dot: "h-3 w-3", text: "text-sm" },
  lg: { dot: "h-4 w-4", text: "text-base" },
};

const variantStyles: Record<StatusIndicatorVariant, { dotClass: string; label: string }> = {
  recording: {
    dotClass: "bg-recording recording-dot",
    label: "Recording",
  },
  processing: {
    dotClass: "bg-processing animate-pulse",
    label: "Processing",
  },
  idle: {
    dotClass: "bg-neutral-400",
    label: "Idle",
  },
};

export const StatusIndicator = forwardRef<HTMLDivElement, StatusIndicatorProps>(
  ({ variant, label, size = "md", className = "", ...props }, ref) => {
    const sizeStyle = sizeClasses[size];
    const variantStyle = variantStyles[variant];
    const displayLabel = label ?? variantStyle.label;

    return (
      <div
        ref={ref}
        className={`inline-flex items-center gap-2 ${className}`}
        role="status"
        aria-label={`Status: ${displayLabel}`}
        {...props}
      >
        <span
          className={`${sizeStyle.dot} rounded-full ${variantStyle.dotClass}`}
          aria-hidden="true"
        />
        {displayLabel && (
          <span className={`${sizeStyle.text} text-text-primary font-medium`}>
            {displayLabel}
          </span>
        )}
      </div>
    );
  }
);

StatusIndicator.displayName = "StatusIndicator";

// Simple recording dot without label
export interface RecordingDotProps extends HTMLAttributes<HTMLSpanElement> {
  active?: boolean;
  size?: "sm" | "md" | "lg";
}

export const RecordingDot = forwardRef<HTMLSpanElement, RecordingDotProps>(
  ({ active = true, size = "md", className = "", ...props }, ref) => {
    const sizeStyle = sizeClasses[size];

    return (
      <span
        ref={ref}
        className={`
          ${sizeStyle.dot}
          rounded-full
          ${active ? "bg-recording recording-dot" : "bg-neutral-400"}
          ${className}
        `}
        aria-hidden="true"
        {...props}
      />
    );
  }
);

RecordingDot.displayName = "RecordingDot";
