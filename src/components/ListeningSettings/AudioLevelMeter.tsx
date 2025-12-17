import "./AudioLevelMeter.css";

export interface AudioLevelMeterProps {
  /** Current audio level (0-100) */
  level: number;
  /** Whether the monitor is currently active */
  isMonitoring: boolean;
}

/**
 * Visual audio level meter component.
 *
 * Displays a horizontal bar showing the current input level with color zones:
 * - Green (0-50): Safe zone, normal speaking levels
 * - Yellow (50-85): Optimal zone, good recording level
 * - Red (85-100): Clipping zone, too loud
 */
export function AudioLevelMeter({
  level,
  isMonitoring,
}: AudioLevelMeterProps): JSX.Element {
  // Determine color zone based on level
  const getZoneClass = (level: number): string => {
    if (level > 85) return "audio-level-meter__fill--clipping";
    if (level > 50) return "audio-level-meter__fill--optimal";
    return "audio-level-meter__fill--safe";
  };

  return (
    <div className="audio-level-meter">
      <div className="audio-level-meter__track">
        <div
          className={`audio-level-meter__fill ${getZoneClass(level)}`}
          style={{ width: `${level}%` }}
          role="progressbar"
          aria-valuenow={level}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label="Audio input level"
        />
        {/* Zone markers */}
        <div
          className="audio-level-meter__marker audio-level-meter__marker--optimal"
          aria-hidden="true"
        />
        <div
          className="audio-level-meter__marker audio-level-meter__marker--clipping"
          aria-hidden="true"
        />
      </div>
      <div className="audio-level-meter__status">
        {isMonitoring ? (
          <span className="audio-level-meter__status--active">Monitoring</span>
        ) : (
          <span className="audio-level-meter__status--idle">Idle</span>
        )}
      </div>
    </div>
  );
}
