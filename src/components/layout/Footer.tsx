import { type ReactNode } from "react";
import { PawPrint } from "lucide-react";

export interface FooterProps {
  /** Left section: current state description */
  stateDescription?: string;
  /** Center section: custom content (e.g., audio meter) */
  center?: ReactNode;
  /** Right section: quick action buttons */
  actions?: ReactNode;
}

export function Footer({
  stateDescription = "Ready for your command.",
  center,
  actions,
}: FooterProps) {
  return (
    <footer
      className="
        h-11 shrink-0
        flex items-center justify-between
        px-4
        border-t border-border
        bg-surface
      "
      role="contentinfo"
    >
      {/* Left: State Description */}
      <div className="flex-1 min-w-0">
        <span className="text-sm text-text-secondary truncate">
          {stateDescription}
        </span>
      </div>

      {/* Center: Audio meter or custom content */}
      {center && (
        <div className="flex-shrink-0 px-4">
          {center}
        </div>
      )}

      {/* Right: Quick Actions + Decorative Paw */}
      <div className="flex-1 min-w-0 flex items-center justify-end gap-3">
        {actions}
        <PawPrint
          className="w-5 h-5 text-heycat-orange/50"
          aria-hidden="true"
        />
      </div>
    </footer>
  );
}
