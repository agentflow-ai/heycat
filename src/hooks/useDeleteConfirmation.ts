import { useState, useCallback } from "react";

/**
 * Configuration options for useDeleteConfirmation hook.
 */
export interface UseDeleteConfirmationOptions {
  /** Callback when delete is confirmed */
  onConfirm?: (id: string) => Promise<void> | void;
}

/**
 * Return type of the useDeleteConfirmation hook.
 */
export interface UseDeleteConfirmationReturn {
  /** ID of the item awaiting delete confirmation, or null */
  confirmingId: string | null;
  /** Request deletion of an item (shows confirmation) */
  requestDelete: (id: string) => void;
  /** Confirm the pending deletion */
  confirmDelete: () => Promise<void>;
  /** Cancel the pending deletion */
  cancelDelete: () => void;
  /** Check if a specific item is awaiting confirmation */
  isConfirming: (id: string) => boolean;
  /** Whether a confirmation is currently pending */
  isPending: boolean;
}

/**
 * Hook for managing delete confirmation state.
 *
 * Provides a two-step deletion pattern where users must confirm
 * before items are deleted.
 *
 * @example
 * const deletion = useDeleteConfirmation({
 *   onConfirm: async (id) => {
 *     await deleteItem(id);
 *     toast.success("Deleted!");
 *   },
 * });
 *
 * // Request deletion
 * <Button onClick={() => deletion.requestDelete(item.id)}>Delete</Button>
 *
 * // Show confirmation UI
 * {deletion.isConfirming(item.id) && (
 *   <>
 *     <Button onClick={deletion.confirmDelete}>Confirm</Button>
 *     <Button onClick={deletion.cancelDelete}>Cancel</Button>
 *   </>
 * )}
 */
export function useDeleteConfirmation(
  options: UseDeleteConfirmationOptions = {}
): UseDeleteConfirmationReturn {
  const { onConfirm } = options;

  const [confirmingId, setConfirmingId] = useState<string | null>(null);

  const requestDelete = useCallback((id: string) => {
    setConfirmingId(id);
  }, []);

  const confirmDelete = useCallback(async () => {
    if (!confirmingId) return;

    try {
      if (onConfirm) {
        await onConfirm(confirmingId);
      }
    } finally {
      // Always clear confirming state, even if onConfirm throws
      setConfirmingId(null);
    }
  }, [confirmingId, onConfirm]);

  const cancelDelete = useCallback(() => {
    setConfirmingId(null);
  }, []);

  const isConfirming = useCallback(
    (id: string): boolean => {
      return confirmingId === id;
    },
    [confirmingId]
  );

  const isPending = confirmingId !== null;

  return {
    confirmingId,
    requestDelete,
    confirmDelete,
    cancelDelete,
    isConfirming,
    isPending,
  };
}
