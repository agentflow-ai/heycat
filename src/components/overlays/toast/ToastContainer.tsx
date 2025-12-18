/**
 * ToastContainer component
 * Source of Truth: ui.md Part 3.7 (Notifications & Toasts)
 *
 * - Fixed position: bottom-right of viewport
 * - Stacks multiple toasts vertically (newest on top)
 * - Z-index above content but below modals
 * - Max 3 visible toasts, older ones dismissed
 */

import { Toast } from "./Toast";
import type { Toast as ToastType } from "./types";

export interface ToastContainerProps {
  toasts: ToastType[];
  onDismiss: (id: string) => void;
}

const MAX_VISIBLE_TOASTS = 3;

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  // Show only the most recent toasts (newest first)
  const visibleToasts = toasts.slice(0, MAX_VISIBLE_TOASTS);

  if (visibleToasts.length === 0) {
    return null;
  }

  return (
    <div
      className="
        fixed bottom-4 right-4
        flex flex-col-reverse gap-3
        z-40
      "
      aria-live="polite"
      aria-label="Notifications"
      data-testid="toast-container"
    >
      {visibleToasts.map((toast) => (
        <Toast key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </div>
  );
}
