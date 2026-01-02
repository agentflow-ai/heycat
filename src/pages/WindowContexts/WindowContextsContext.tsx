import { createContext, useContext, useMemo, useCallback, useState, type ReactNode } from "react";
import { useToast } from "../../components/overlays";
import { useWindowContext, useRunningApplications } from "../../hooks/useWindowContext";
import { useDictionary } from "../../hooks/useDictionary";
import { useSearch } from "../../hooks/useSearch";
import { useDeleteConfirmation } from "../../hooks/useDeleteConfirmation";
import type { WindowContext, OverrideMode, RunningApplication } from "../../types/windowContext";
import type { DictionaryEntry } from "../../types/dictionary";
import type { ComboboxOption, MultiSelectOption } from "../../components/ui";
import { validateRegexPattern } from "../../lib/validation";

/**
 * Edit state for a window context.
 */
export interface EditState {
  name: string;
  appName: string;
  bundleId: string | undefined;
  titlePattern: string;
  commandMode: OverrideMode;
  dictionaryMode: OverrideMode;
  dictionaryEntryIds: string[];
  priority: number;
  enabled: boolean;
}

/**
 * Context value for window contexts state and actions.
 */
export interface WindowContextsContextValue {
  // Data
  contextList: WindowContext[];
  filteredContexts: WindowContext[];
  runningApps: RunningApplication[];
  dictionaryEntries: DictionaryEntry[];
  appOptions: ComboboxOption[];
  dictionaryOptions: MultiSelectOption[];

  // Search
  searchQuery: string;
  setSearchQuery: (query: string) => void;

  // Edit state
  editingId: string | null;
  editValues: EditState;
  editError: string | null;
  patternError: string | null;

  // Delete state
  deletion: {
    confirmingId: string | null;
    requestDelete: (id: string) => void;
    confirmDelete: () => Promise<void>;
    cancelDelete: () => void;
    isConfirming: (id: string) => boolean;
  };

  // Actions
  handleAddContext: (data: {
    name: string;
    appName: string;
    bundleId?: string;
    titlePattern?: string;
    commandMode: OverrideMode;
    dictionaryMode: OverrideMode;
    dictionaryEntryIds: string[];
    priority: number;
    enabled: boolean;
  }) => Promise<void>;
  handleStartEdit: (ctx: WindowContext) => void;
  handleEditChange: (
    field: keyof EditState,
    value: string | boolean | string[] | number | undefined
  ) => void;
  handleSaveEdit: () => Promise<void>;
  handleCancelEdit: () => void;
  handleToggleEnabled: (id: string, enabled: boolean) => Promise<void>;

  // Loading/error states
  isLoading: boolean;
  isError: boolean;
  error: Error | null;
  refetch: () => void;
}

const WindowContextsContext = createContext<WindowContextsContextValue | null>(null);

/**
 * Hook to access window contexts context.
 */
export function useWindowContextsContext(): WindowContextsContextValue {
  const context = useContext(WindowContextsContext);
  if (!context) {
    throw new Error("useWindowContextsContext must be used within a WindowContextsProvider");
  }
  return context;
}

interface WindowContextsProviderProps {
  children: ReactNode;
}

const DEFAULT_EDIT_STATE: EditState = {
  name: "",
  appName: "",
  bundleId: undefined,
  titlePattern: "",
  commandMode: "merge",
  dictionaryMode: "merge",
  dictionaryEntryIds: [],
  priority: 0,
  enabled: true,
};

/**
 * Provider for window contexts page state.
 */
export function WindowContextsProvider({ children }: WindowContextsProviderProps) {
  const { toast } = useToast();
  const { contexts, addContext, updateContext, deleteContext } = useWindowContext();
  const runningAppsQuery = useRunningApplications();
  const { entries: dictionaryEntriesQuery } = useDictionary();

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState<EditState>(DEFAULT_EDIT_STATE);
  const [editError, setEditError] = useState<string | null>(null);
  const [patternError, setPatternError] = useState<string | null>(null);

  const contextList = Array.isArray(contexts.data) ? contexts.data : [];
  const runningApps = runningAppsQuery.data ?? [];
  const dictionaryEntries = dictionaryEntriesQuery.data ?? [];

  // Use the search hook
  const {
    query: searchQuery,
    setQuery: setSearchQuery,
    filteredItems: filteredContexts,
  } = useSearch({
    items: contextList,
    getSearchableText: (ctx) =>
      `${ctx.name} ${ctx.matcher.appName} ${ctx.matcher.titlePattern ?? ""}`,
  });

  // Use the delete confirmation hook
  const deletion = useDeleteConfirmation({
    onConfirm: async (id) => {
      const ctx = contextList.find((c) => c.id === id);
      try {
        await deleteContext.mutateAsync({ id });
        toast({
          type: "success",
          title: "Context deleted",
          description: ctx ? `"${ctx.name}" has been removed.` : "Context removed.",
        });
      } catch (e) {
        toast({
          type: "error",
          title: "Failed to delete context",
          description: e instanceof Error ? e.message : String(e),
        });
      }
    },
  });

  // Convert running apps to combobox options
  const appOptions: ComboboxOption[] = useMemo(
    () =>
      runningApps.map((app) => ({
        label: app.name,
        value: app.name,
        description: app.bundleId,
      })),
    [runningApps]
  );

  // Convert dictionary entries to multiselect options
  const dictionaryOptions: MultiSelectOption[] = useMemo(
    () =>
      dictionaryEntries.map((entry) => ({
        value: entry.id,
        label: entry.trigger,
        description: entry.expansion,
      })),
    [dictionaryEntries]
  );

  const handleAddContext = useCallback(
    async (data: {
      name: string;
      appName: string;
      bundleId?: string;
      titlePattern?: string;
      commandMode: OverrideMode;
      dictionaryMode: OverrideMode;
      dictionaryEntryIds: string[];
      priority: number;
      enabled: boolean;
    }) => {
      try {
        await addContext.mutateAsync({
          name: data.name,
          appName: data.appName,
          bundleId: data.bundleId,
          titlePattern: data.titlePattern,
          commandMode: data.commandMode,
          dictionaryMode: data.dictionaryMode,
          dictionaryEntryIds: data.dictionaryEntryIds,
          priority: data.priority,
          enabled: data.enabled,
        });
        toast({
          type: "success",
          title: "Context added",
          description: `"${data.name}" has been created.`,
        });
      } catch (e) {
        toast({
          type: "error",
          title: "Failed to add context",
          description: e instanceof Error ? e.message : String(e),
        });
        throw e;
      }
    },
    [addContext, toast]
  );

  const handleStartEdit = useCallback((ctx: WindowContext) => {
    setEditingId(ctx.id);
    setEditValues({
      name: ctx.name,
      appName: ctx.matcher.appName,
      bundleId: ctx.matcher.bundleId,
      titlePattern: ctx.matcher.titlePattern ?? "",
      commandMode: ctx.commandMode,
      dictionaryMode: ctx.dictionaryMode,
      dictionaryEntryIds: ctx.dictionaryEntryIds,
      priority: ctx.priority,
      enabled: ctx.enabled,
    });
    setEditError(null);
    setPatternError(null);
  }, []);

  const handleEditChange = useCallback(
    (field: keyof EditState, value: string | boolean | string[] | number | undefined) => {
      setEditValues((prev) => ({ ...prev, [field]: value }));
      if (field === "name") {
        setEditError(null);
      }
      if (field === "titlePattern" && typeof value === "string") {
        const error = validateRegexPattern(value);
        setPatternError(error);
      }
    },
    []
  );

  const handleSaveEdit = useCallback(async () => {
    if (!editingId) return;

    const trimmedName = editValues.name.trim();
    if (!trimmedName) {
      setEditError("Name is required");
      return;
    }

    // Validate pattern
    const patternValidationError = validateRegexPattern(editValues.titlePattern);
    if (patternValidationError) {
      setPatternError(patternValidationError);
      return;
    }

    try {
      await updateContext.mutateAsync({
        id: editingId,
        name: trimmedName,
        appName: editValues.appName.trim(),
        bundleId: editValues.bundleId,
        titlePattern: editValues.titlePattern.trim() || undefined,
        commandMode: editValues.commandMode,
        dictionaryMode: editValues.dictionaryMode,
        commandIds: [], // Preserve existing
        dictionaryEntryIds: editValues.dictionaryEntryIds,
        priority: editValues.priority,
        enabled: editValues.enabled,
      });

      toast({
        type: "success",
        title: "Context updated",
        description: `"${trimmedName}" has been updated.`,
      });
      setEditingId(null);
      setEditValues(DEFAULT_EDIT_STATE);
      setEditError(null);
      setPatternError(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to update context",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }, [editingId, editValues, updateContext, toast]);

  const handleCancelEdit = useCallback(() => {
    setEditingId(null);
    setEditValues(DEFAULT_EDIT_STATE);
    setEditError(null);
    setPatternError(null);
  }, []);

  const handleToggleEnabled = useCallback(
    async (id: string, enabled: boolean) => {
      const ctx = contextList.find((c) => c.id === id);
      if (!ctx) return;

      try {
        await updateContext.mutateAsync({
          id,
          name: ctx.name,
          appName: ctx.matcher.appName,
          bundleId: ctx.matcher.bundleId,
          titlePattern: ctx.matcher.titlePattern,
          commandMode: ctx.commandMode,
          dictionaryMode: ctx.dictionaryMode,
          commandIds: ctx.commandIds,
          dictionaryEntryIds: ctx.dictionaryEntryIds,
          priority: ctx.priority,
          enabled,
        });
      } catch (e) {
        toast({
          type: "error",
          title: "Failed to update context",
          description: e instanceof Error ? e.message : String(e),
        });
      }
    },
    [contextList, updateContext, toast]
  );

  const value: WindowContextsContextValue = {
    contextList,
    filteredContexts,
    runningApps,
    dictionaryEntries,
    appOptions,
    dictionaryOptions,
    searchQuery,
    setSearchQuery,
    editingId,
    editValues,
    editError,
    patternError,
    deletion,
    handleAddContext,
    handleStartEdit,
    handleEditChange,
    handleSaveEdit,
    handleCancelEdit,
    handleToggleEnabled,
    isLoading: contexts.isLoading,
    isError: contexts.isError,
    error: contexts.error instanceof Error ? contexts.error : null,
    refetch: contexts.refetch,
  };

  return (
    <WindowContextsContext.Provider value={value}>{children}</WindowContextsContext.Provider>
  );
}
