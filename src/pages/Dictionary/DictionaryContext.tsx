import { createContext, useContext, useMemo, useCallback, useState, type ReactNode } from "react";
import { useToast } from "../../components/overlays";
import { useDictionary } from "../../hooks/useDictionary";
import { useWindowContext } from "../../hooks/useWindowContext";
import { useSearch } from "../../hooks/useSearch";
import { useDeleteConfirmation } from "../../hooks/useDeleteConfirmation";
import type { DictionaryEntry } from "../../types/dictionary";
import type { WindowContext } from "../../types/windowContext";
import type { MultiSelectOption } from "../../components/ui";
import { validateSuffix } from "../../lib/validation";

/**
 * Edit state for a dictionary entry.
 */
export interface EditState {
  trigger: string;
  expansion: string;
  suffix: string;
  autoEnter: boolean;
  disableSuffix: boolean;
  completeMatchOnly: boolean;
  contextIds: string[];
}

/**
 * Context value for dictionary state and actions.
 */
export interface DictionaryContextValue {
  // Data
  entryList: DictionaryEntry[];
  contextList: WindowContext[];
  filteredEntries: DictionaryEntry[];
  existingTriggers: string[];
  contextOptions: MultiSelectOption[];
  contextsByEntryId: Map<string, WindowContext[]>;

  // Search
  searchQuery: string;
  setSearchQuery: (query: string) => void;

  // Edit state
  editingId: string | null;
  editValues: EditState;
  editError: string | null;
  editSuffixError: string | null;

  // Delete state (from useDeleteConfirmation)
  deletion: {
    confirmingId: string | null;
    requestDelete: (id: string) => void;
    confirmDelete: () => Promise<void>;
    cancelDelete: () => void;
    isConfirming: (id: string) => boolean;
  };

  // Actions
  handleAddEntry: (
    trigger: string,
    expansion: string,
    contextIds: string[],
    suffix?: string,
    autoEnter?: boolean,
    disableSuffix?: boolean,
    completeMatchOnly?: boolean
  ) => Promise<void>;
  handleStartEdit: (entry: DictionaryEntry) => void;
  handleEditChange: (
    field: "trigger" | "expansion" | "suffix" | "autoEnter" | "disableSuffix" | "completeMatchOnly" | "contextIds",
    value: string | boolean | string[]
  ) => void;
  handleSaveEdit: () => Promise<void>;
  handleCancelEdit: () => void;

  // Loading/error states
  isLoading: boolean;
  isError: boolean;
  error: Error | null;
  refetch: () => void;
}

const DictionaryContext = createContext<DictionaryContextValue | null>(null);

/**
 * Hook to access dictionary context.
 * Must be used within a DictionaryProvider.
 */
export function useDictionaryContext(): DictionaryContextValue {
  const context = useContext(DictionaryContext);
  if (!context) {
    throw new Error("useDictionaryContext must be used within a DictionaryProvider");
  }
  return context;
}

interface DictionaryProviderProps {
  children: ReactNode;
}

const DEFAULT_EDIT_STATE: EditState = {
  trigger: "",
  expansion: "",
  suffix: "",
  autoEnter: false,
  disableSuffix: false,
  completeMatchOnly: false,
  contextIds: [],
};

/**
 * Provider for dictionary page state.
 */
export function DictionaryProvider({ children }: DictionaryProviderProps) {
  const { toast } = useToast();
  const { entries, addEntry, updateEntry, deleteEntry } = useDictionary();
  const { contexts, updateContext } = useWindowContext();

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState<EditState>(DEFAULT_EDIT_STATE);
  const [editError, setEditError] = useState<string | null>(null);
  const [editSuffixError, setEditSuffixError] = useState<string | null>(null);

  const entryList = Array.isArray(entries.data) ? entries.data : [];
  const contextList = Array.isArray(contexts.data) ? contexts.data : [];

  // Use the search hook for filtering entries
  const {
    query: searchQuery,
    setQuery: setSearchQuery,
    filteredItems: filteredEntries,
  } = useSearch({
    items: entryList,
    getSearchableText: (entry) => `${entry.trigger} ${entry.expansion}`,
  });

  // Use the delete confirmation hook
  const deletion = useDeleteConfirmation({
    onConfirm: async (id) => {
      const entry = entryList.find((e) => e.id === id);
      try {
        await deleteEntry.mutateAsync({ id });
        toast({
          type: "success",
          title: "Entry deleted",
          description: entry ? `"${entry.trigger}" has been removed.` : "Entry removed.",
        });
      } catch (e) {
        toast({
          type: "error",
          title: "Failed to delete entry",
          description: e instanceof Error ? e.message : String(e),
        });
      }
    },
  });

  // Reverse lookup: entry ID -> contexts that include it
  const contextsByEntryId = useMemo(() => {
    const map = new Map<string, WindowContext[]>();
    for (const ctx of contextList) {
      for (const entryId of ctx.dictionaryEntryIds) {
        const existing = map.get(entryId) ?? [];
        existing.push(ctx);
        map.set(entryId, existing);
      }
    }
    return map;
  }, [contextList]);

  // Convert contexts to MultiSelect options (only enabled contexts)
  const contextOptions: MultiSelectOption[] = useMemo(
    () =>
      contextList
        .filter((ctx) => ctx.enabled)
        .map((ctx) => ({
          value: ctx.id,
          label: ctx.name,
          description: ctx.matcher?.appName,
        })),
    [contextList]
  );

  const existingTriggers = useMemo(
    () => entryList.map((e) => e.trigger.toLowerCase()),
    [entryList]
  );

  const handleAddEntry = useCallback(
    async (
      trigger: string,
      expansion: string,
      contextIds: string[],
      suffix?: string,
      autoEnter?: boolean,
      disableSuffix?: boolean,
      completeMatchOnly?: boolean
    ) => {
      try {
        const newEntry = await addEntry.mutateAsync({
          trigger,
          expansion,
          suffix,
          autoEnter,
          disableSuffix,
          completeMatchOnly,
        });

        // Update selected contexts to include the new entry
        for (const ctxId of contextIds) {
          const ctx = contextList.find((c) => c.id === ctxId);
          if (ctx) {
            await updateContext.mutateAsync({
              id: ctx.id,
              name: ctx.name,
              appName: ctx.matcher?.appName,
              titlePattern: ctx.matcher?.titlePattern,
              bundleId: ctx.matcher?.bundleId,
              commandMode: ctx.commandMode,
              dictionaryMode: ctx.dictionaryMode,
              commandIds: ctx.commandIds,
              dictionaryEntryIds: [...ctx.dictionaryEntryIds, newEntry.id],
              priority: ctx.priority,
              enabled: ctx.enabled,
            });
          }
        }

        toast({
          type: "success",
          title: "Entry added",
          description: `"${trigger}" has been added to your dictionary.`,
        });
      } catch (e) {
        toast({
          type: "error",
          title: "Failed to add entry",
          description: e instanceof Error ? e.message : String(e),
        });
        throw e;
      }
    },
    [addEntry, contextList, updateContext, toast]
  );

  const handleStartEdit = useCallback(
    (entry: DictionaryEntry) => {
      setEditingId(entry.id);
      // Get current context assignments for this entry
      const assignedContextIds = (contextsByEntryId.get(entry.id) ?? []).map((ctx) => ctx.id);
      setEditValues({
        trigger: entry.trigger,
        expansion: entry.expansion,
        suffix: entry.suffix || "",
        autoEnter: entry.autoEnter || false,
        disableSuffix: entry.disableSuffix || false,
        completeMatchOnly: entry.completeMatchOnly || false,
        contextIds: assignedContextIds,
      });
      setEditError(null);
      setEditSuffixError(null);
    },
    [contextsByEntryId]
  );

  const handleEditChange = useCallback(
    (
      field: "trigger" | "expansion" | "suffix" | "autoEnter" | "disableSuffix" | "completeMatchOnly" | "contextIds",
      value: string | boolean | string[]
    ) => {
      setEditValues((prev) => ({ ...prev, [field]: value }));
      if (field === "trigger") {
        setEditError(null);
      }
      if (field === "suffix" && typeof value === "string") {
        const error = validateSuffix(value);
        setEditSuffixError(error);
      }
    },
    []
  );

  const handleSaveEdit = useCallback(async () => {
    if (!editingId) return;

    const trimmedTrigger = editValues.trigger.trim();
    if (!trimmedTrigger) {
      setEditError("Trigger is required");
      return;
    }

    // Check for duplicate, excluding the current entry
    const isDuplicate = entryList.some(
      (e) => e.id !== editingId && e.trigger.toLowerCase() === trimmedTrigger.toLowerCase()
    );
    if (isDuplicate) {
      setEditError("This trigger already exists");
      return;
    }

    // Validate suffix
    const suffixError = validateSuffix(editValues.suffix);
    if (suffixError) {
      setEditSuffixError(suffixError);
      return;
    }

    try {
      await updateEntry.mutateAsync({
        id: editingId,
        trigger: trimmedTrigger,
        expansion: editValues.expansion.trim(),
        suffix: editValues.disableSuffix ? undefined : editValues.suffix.trim() || undefined,
        autoEnter: editValues.autoEnter || undefined,
        disableSuffix: editValues.disableSuffix || undefined,
        completeMatchOnly: editValues.completeMatchOnly || undefined,
      });

      // Sync context assignments
      const oldContextIds = (contextsByEntryId.get(editingId) ?? []).map((ctx) => ctx.id);
      const newContextIds = editValues.contextIds;

      // Contexts to add entry to
      const contextsToAdd = newContextIds.filter((id) => !oldContextIds.includes(id));
      // Contexts to remove entry from
      const contextsToRemove = oldContextIds.filter((id) => !newContextIds.includes(id));

      for (const ctxId of contextsToAdd) {
        const ctx = contextList.find((c) => c.id === ctxId);
        if (ctx) {
          await updateContext.mutateAsync({
            id: ctx.id,
            name: ctx.name,
            appName: ctx.matcher?.appName,
            titlePattern: ctx.matcher?.titlePattern,
            bundleId: ctx.matcher?.bundleId,
            commandMode: ctx.commandMode,
            dictionaryMode: ctx.dictionaryMode,
            commandIds: ctx.commandIds,
            dictionaryEntryIds: [...ctx.dictionaryEntryIds, editingId],
            priority: ctx.priority,
            enabled: ctx.enabled,
          });
        }
      }

      for (const ctxId of contextsToRemove) {
        const ctx = contextList.find((c) => c.id === ctxId);
        if (ctx) {
          await updateContext.mutateAsync({
            id: ctx.id,
            name: ctx.name,
            appName: ctx.matcher?.appName,
            titlePattern: ctx.matcher?.titlePattern,
            bundleId: ctx.matcher?.bundleId,
            commandMode: ctx.commandMode,
            dictionaryMode: ctx.dictionaryMode,
            commandIds: ctx.commandIds,
            dictionaryEntryIds: ctx.dictionaryEntryIds.filter((id) => id !== editingId),
            priority: ctx.priority,
            enabled: ctx.enabled,
          });
        }
      }

      toast({
        type: "success",
        title: "Entry updated",
        description: `"${trimmedTrigger}" has been updated.`,
      });
      setEditingId(null);
      setEditValues(DEFAULT_EDIT_STATE);
      setEditError(null);
      setEditSuffixError(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to update entry",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }, [editingId, editValues, entryList, updateEntry, toast, contextsByEntryId, contextList, updateContext]);

  const handleCancelEdit = useCallback(() => {
    setEditingId(null);
    setEditValues(DEFAULT_EDIT_STATE);
    setEditError(null);
    setEditSuffixError(null);
  }, []);

  const value: DictionaryContextValue = {
    entryList,
    contextList,
    filteredEntries,
    existingTriggers,
    contextOptions,
    contextsByEntryId,
    searchQuery,
    setSearchQuery,
    editingId,
    editValues,
    editError,
    editSuffixError,
    deletion,
    handleAddEntry,
    handleStartEdit,
    handleEditChange,
    handleSaveEdit,
    handleCancelEdit,
    isLoading: entries.isLoading,
    isError: entries.isError,
    error: entries.error instanceof Error ? entries.error : null,
    refetch: entries.refetch,
  };

  return <DictionaryContext.Provider value={value}>{children}</DictionaryContext.Provider>;
}
