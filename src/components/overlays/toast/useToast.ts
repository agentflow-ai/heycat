/**
 * useToast hook - Access toast functionality from any component
 * Source of Truth: ui.md Part 3.7 (Notifications & Toasts)
 *
 * Usage:
 * ```tsx
 * const { toast, dismiss, dismissAll } = useToast();
 *
 * // Show a success toast
 * toast({
 *   type: 'success',
 *   title: 'Transcription complete',
 *   description: 'Hello, this is...',
 *   action: {
 *     label: 'Copy to Clipboard',
 *     onClick: () => copyToClipboard(text)
 *   }
 * });
 *
 * // Show an error toast (won't auto-dismiss)
 * toast({
 *   type: 'error',
 *   title: 'Recording failed',
 *   description: 'Microphone not available'
 * });
 * ```
 */

import { useContext } from "react";
import { ToastContext } from "./ToastProvider";
import type { ToastContextValue } from "./types";

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}
