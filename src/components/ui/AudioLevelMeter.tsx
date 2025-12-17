import { type HTMLAttributes, forwardRef, useMemo } from "react";

export interface AudioLevelMeterProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * Audio level from 0 to 100
   */
  level: number;
  /**
   * Optional threshold for optimal zone (default: 30)
   */
  optimalThreshold?: number;
  /**
   * Optional threshold for clipping zone (default: 85)
   */
  clippingThreshold?: number;
  /**
   * Orientation of the meter
   */
  orientation?: "horizontal" | "vertical";
  /**
   * Show level text
   */
  showValue?: boolean;
}

export const AudioLevelMeter = forwardRef<HTMLDivElement, AudioLevelMeterProps>(
  (
    {
      level,
      optimalThreshold = 30,
      clippingThreshold = 85,
      orientation = "horizontal",
      showValue = false,
      className = "",
      ...props
    },
    ref
  ) => {
    // Clamp level between 0 and 100
    const clampedLevel = Math.max(0, Math.min(100, level));

    // Determine color based on thresholds
    const levelColor = useMemo(() => {
      if (clampedLevel >= clippingThreshold) return "bg-error"; // Red - clipping
      if (clampedLevel >= optimalThreshold) return "bg-success"; // Green - optimal
      return "bg-success"; // Green - safe (below optimal is fine)
    }, [clampedLevel, optimalThreshold, clippingThreshold]);

    const isHorizontal = orientation === "horizontal";

    return (
      <div
        ref={ref}
        className={`
          relative
          ${isHorizontal ? "h-2 w-full" : "w-2 h-full"}
          bg-neutral-200
          rounded-full
          overflow-hidden
          ${className}
        `}
        role="meter"
        aria-valuenow={clampedLevel}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Audio level"
        {...props}
      >
        {/* Fill bar */}
        <div
          className={`
            absolute
            ${isHorizontal ? "h-full left-0 top-0" : "w-full bottom-0 left-0"}
            ${levelColor}
            rounded-full
            transition-all duration-75 ease-out
          `}
          style={
            isHorizontal
              ? { width: `${clampedLevel}%` }
              : { height: `${clampedLevel}%` }
          }
        />

        {/* Threshold markers for horizontal orientation */}
        {isHorizontal && (
          <>
            <div
              className="absolute top-0 bottom-0 w-px bg-neutral-400/50"
              style={{ left: `${optimalThreshold}%` }}
              aria-hidden="true"
            />
            <div
              className="absolute top-0 bottom-0 w-px bg-error/50"
              style={{ left: `${clippingThreshold}%` }}
              aria-hidden="true"
            />
          </>
        )}
      </div>
    );
  }
);

AudioLevelMeter.displayName = "AudioLevelMeter";

// Simplified inline meter for status bars
export interface MiniAudioMeterProps {
  level: number;
  className?: string;
}

export const MiniAudioMeter = ({ level, className = "" }: MiniAudioMeterProps) => {
  const clampedLevel = Math.max(0, Math.min(100, level));
  const isClipping = clampedLevel >= 85;

  return (
    <div
      className={`inline-flex items-center gap-0.5 h-3 ${className}`}
      role="meter"
      aria-valuenow={clampedLevel}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label="Audio level"
    >
      {/* 5 bars visualization */}
      {[20, 40, 60, 80, 100].map((threshold) => (
        <div
          key={threshold}
          className={`
            w-1 rounded-sm
            ${clampedLevel >= threshold
              ? isClipping && threshold > 80
                ? "bg-error"
                : "bg-success"
              : "bg-neutral-300"
            }
          `}
          style={{ height: `${40 + threshold * 0.6}%` }}
          aria-hidden="true"
        />
      ))}
    </div>
  );
};
