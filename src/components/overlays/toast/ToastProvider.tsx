/**
 * ToastProvider - Context provider for toast notifications
 * Source of Truth: ui.md Part 3.7 (Notifications & Toasts)
 *
 * Usage:
 * ```tsx
 * // Wrap app root with ToastProvider
 * <ToastProvider>
 *   <App />
 * </ToastProvider>
 *
 * // In components
 * const { toast } = useToast();
 * toast({
 *   type: 'success',
 *   title: 'Transcription complete',
 *   description: 'Hello, this is...',
 *   action: {
 *     label: 'Copy to Clipboard',
 *     onClick: () => copyToClipboard(text)
 *   }
 * });
 * ```
 */

import {
  createContext,
  useCallback,
  useState,
  type ReactNode,
} from "react";
import { ToastContainer } from "./ToastContainer";
import type { Toast, ToastOptions, ToastContextValue } from "./types";

export const ToastContext = createContext<ToastContextValue | null>(null);

let toastIdCounter = 0;
const generateId = () => `toast-${++toastIdCounter}`;

export interface ToastProviderProps {
  children: ReactNode;
}

export function ToastProvider({ children }: ToastProviderProps) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = useCallback((options: ToastOptions): string => {
    const id = generateId();
    const newToast: Toast = {
      id,
      type: options.type,
      title: options.title,
      description: options.description,
      action: options.action,
      // Errors don't auto-dismiss by default, others do after 5s
      duration:
        options.duration !== undefined
          ? options.duration
          : options.type === "error"
            ? null
            : 5000,
    };

    // Add to beginning (newest first)
    setToasts((prev) => [newToast, ...prev]);
    return id;
  }, []);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const dismissAll = useCallback(() => {
    setToasts([]);
  }, []);

  const contextValue: ToastContextValue = {
    toasts,
    toast,
    dismiss,
    dismissAll,
  };

  return (
    <ToastContext.Provider value={contextValue}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={dismiss} />
    </ToastContext.Provider>
  );
}
