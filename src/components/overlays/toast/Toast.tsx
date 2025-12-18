/**
 * Toast component
 * Source of Truth: ui.md Part 3.7 (Notifications & Toasts)
 *
 * Layout:
 * +------------------------------------------+
 * | [Check icon]  Transcription complete     |
 * |               "Hello, this is..."   [X]  |
 * |               [Copy to Clipboard]        |
 * +------------------------------------------+
 */

import { forwardRef, useEffect, useRef, useState, useCallback } from "react";
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  Info,
  X,
} from "lucide-react";
import { Button } from "../../ui";
import type { Toast as ToastType, ToastType as ToastVariant } from "./types";

export interface ToastProps {
  toast: ToastType;
  onDismiss: (id: string) => void;
}

const iconMap: Record<ToastVariant, typeof CheckCircle> = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
};

const typeStyles: Record<ToastVariant, { icon: string; border: string }> = {
  success: {
    icon: "text-success",
    border: "border-l-success",
  },
  error: {
    icon: "text-error",
    border: "border-l-error",
  },
  warning: {
    icon: "text-warning",
    border: "border-l-warning",
  },
  info: {
    icon: "text-info",
    border: "border-l-info",
  },
};

export const Toast = forwardRef<HTMLDivElement, ToastProps>(
  ({ toast, onDismiss }, ref) => {
    const { id, type, title, description, action, duration } = toast;
    const [isPaused, setIsPaused] = useState(false);
    const [isExiting, setIsExiting] = useState(false);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const remainingTimeRef = useRef(duration ?? 5000);
    const startTimeRef = useRef<number | null>(null);

    const Icon = iconMap[type];
    const styles = typeStyles[type];

    const handleDismiss = useCallback(() => {
      setIsExiting(true);
      // Wait for exit animation before removing
      setTimeout(() => {
        onDismiss(id);
      }, 200);
    }, [id, onDismiss]);

    // Auto-dismiss logic with pause on hover
    useEffect(() => {
      // No auto-dismiss for errors or if duration is null
      if (type === "error" || duration === null) {
        return;
      }

      const startTimer = () => {
        startTimeRef.current = Date.now();
        timerRef.current = setTimeout(handleDismiss, remainingTimeRef.current);
      };

      const pauseTimer = () => {
        if (timerRef.current) {
          clearTimeout(timerRef.current);
          timerRef.current = null;
          if (startTimeRef.current) {
            remainingTimeRef.current -= Date.now() - startTimeRef.current;
          }
        }
      };

      if (!isPaused) {
        startTimer();
      } else {
        pauseTimer();
      }

      return () => {
        if (timerRef.current) {
          clearTimeout(timerRef.current);
        }
      };
    }, [isPaused, type, duration, handleDismiss]);

    return (
      <div
        ref={ref}
        role="alert"
        aria-live={type === "error" ? "assertive" : "polite"}
        onMouseEnter={() => setIsPaused(true)}
        onMouseLeave={() => setIsPaused(false)}
        className={`
          w-[360px]
          bg-surface
          rounded-[var(--radius-lg)]
          shadow-lg
          border border-border
          border-l-4 ${styles.border}
          transition-all duration-[var(--duration-normal)] ease-[var(--ease-default)]
          ${isExiting
            ? "translate-x-full opacity-0"
            : "translate-x-0 opacity-100 animate-slide-in"
          }
        `}
        data-testid="toast"
        data-toast-type={type}
      >
        <div className="flex items-start gap-3 p-4">
          {/* Icon */}
          <Icon
            className={`w-5 h-5 shrink-0 mt-0.5 ${styles.icon}`}
            aria-hidden="true"
          />

          {/* Content */}
          <div className="flex-1 min-w-0">
            <p className="font-medium text-text-primary">{title}</p>
            {description && (
              <p className="mt-1 text-sm text-text-secondary line-clamp-2">
                {description}
              </p>
            )}
            {action && (
              <div className="mt-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={action.onClick}
                  className="text-heycat-teal hover:text-heycat-teal-dark"
                >
                  {action.label}
                </Button>
              </div>
            )}
          </div>

          {/* Close button */}
          <button
            onClick={handleDismiss}
            className="
              shrink-0
              p-1
              rounded
              text-text-secondary
              hover:text-text-primary
              hover:bg-text-secondary/10
              transition-colors duration-[var(--duration-fast)]
            "
            aria-label="Dismiss notification"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    );
  }
);

Toast.displayName = "Toast";
