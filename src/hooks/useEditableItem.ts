import { useState, useCallback } from "react";

/**
 * Configuration options for useEditableItem hook.
 */
export interface UseEditableItemOptions<T, V extends Record<string, unknown> = Record<string, unknown>> {
  /** Function to get initial edit values from an item */
  getInitialValues: (item: T) => V;
}

/**
 * Return type of the useEditableItem hook.
 */
export interface UseEditableItemReturn<T, V extends Record<string, unknown>> {
  /** ID of the item currently being edited, or null */
  editingId: string | null;
  /** Current edit values */
  editValues: V;
  /** Start editing an item */
  startEdit: (id: string, item: T) => void;
  /** Cancel editing */
  cancelEdit: () => void;
  /** Check if a specific item is being edited */
  isEditing: (id: string) => boolean;
  /** Update edit values */
  setEditValues: React.Dispatch<React.SetStateAction<V>>;
  /** Update a single edit value field */
  setEditValue: <K extends keyof V>(field: K, value: V[K]) => void;
}

/**
 * Hook for managing inline editing of items in a list.
 *
 * Tracks which item is being edited and maintains the edit values
 * separately from the original data.
 *
 * @example
 * const editable = useEditableItem<DictionaryEntry, EditValues>({
 *   getInitialValues: (entry) => ({
 *     trigger: entry.trigger,
 *     expansion: entry.expansion,
 *   }),
 * });
 *
 * // Start editing
 * editable.startEdit(entry.id, entry);
 *
 * // Check if editing
 * if (editable.isEditing(entry.id)) {
 *   // Render edit form with editable.editValues
 * }
 */
export function useEditableItem<T, V extends Record<string, unknown>>(
  options: UseEditableItemOptions<T, V>
): UseEditableItemReturn<T, V> {
  const { getInitialValues } = options;

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState<V>({} as V);

  const startEdit = useCallback(
    (id: string, item: T) => {
      setEditingId(id);
      setEditValues(getInitialValues(item) as V);
    },
    [getInitialValues]
  );

  const cancelEdit = useCallback(() => {
    setEditingId(null);
    setEditValues({} as V);
  }, []);

  const isEditing = useCallback(
    (id: string): boolean => {
      return editingId === id;
    },
    [editingId]
  );

  const setEditValue = useCallback(<K extends keyof V>(field: K, value: V[K]) => {
    setEditValues((prev) => ({ ...prev, [field]: value }));
  }, []);

  return {
    editingId,
    editValues,
    startEdit,
    cancelEdit,
    isEditing,
    setEditValues,
    setEditValue,
  };
}
