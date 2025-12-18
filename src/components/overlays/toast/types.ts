/**
 * Toast notification types
 * Source of Truth: ui.md Part 3.7 (Notifications & Toasts)
 */

export type ToastType = "success" | "error" | "warning" | "info";

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  /** Unique identifier for the toast */
  id: string;
  /** Type determines icon and color styling */
  type: ToastType;
  /** Bold title text */
  title: string;
  /** Optional description text (can be truncated if long) */
  description?: string;
  /** Optional action buttons */
  action?: ToastAction;
  /** Duration in ms before auto-dismiss (default: 5000, null = no auto-dismiss) */
  duration?: number | null;
}

export interface ToastOptions {
  type: ToastType;
  title: string;
  description?: string;
  action?: ToastAction;
  /** Duration in ms (default: 5000, null = no auto-dismiss, errors default to null) */
  duration?: number | null;
}

export interface ToastContextValue {
  toasts: Toast[];
  toast: (options: ToastOptions) => string;
  dismiss: (id: string) => void;
  dismissAll: () => void;
}
