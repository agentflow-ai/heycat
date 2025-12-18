/**
 * Toast notification system (ui.md 3.7)
 *
 * Integration Point:
 * - Wrapped at app root in ToastProvider
 * - Connects to: useTranscription (transcription results), error handlers
 */

// Components
export { Toast } from "./Toast";
export type { ToastProps } from "./Toast";

export { ToastContainer } from "./ToastContainer";
export type { ToastContainerProps } from "./ToastContainer";

export { ToastProvider } from "./ToastProvider";
export type { ToastProviderProps } from "./ToastProvider";

// Hook
export { useToast } from "./useToast";

// Types
export type {
  Toast as ToastType,
  ToastType as ToastVariant,
  ToastAction,
  ToastOptions,
  ToastContextValue,
} from "./types";
