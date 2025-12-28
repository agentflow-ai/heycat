import { useState, useMemo, useCallback } from "react";
import { Plus, Search, Layers, Pencil, Trash2, Check, X, BookText } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, Toggle, Combobox, MultiSelect } from "../components/ui";
import type { ComboboxOption, MultiSelectOption } from "../components/ui";
import { useToast } from "../components/overlays";
import { useWindowContext, useRunningApplications } from "../hooks/useWindowContext";
import { useDictionary } from "../hooks/useDictionary";
import type { WindowContext, OverrideMode, RunningApplication } from "../types/windowContext";
import type { DictionaryEntry } from "../types/dictionary";

export interface WindowContextsProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

interface AddContextFormProps {
  onSubmit: (data: {
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
  runningApps: RunningApplication[];
  dictionaryEntries: DictionaryEntry[];
}

function AddContextForm({ onSubmit, runningApps, dictionaryEntries }: AddContextFormProps) {
  const [name, setName] = useState("");
  const [appName, setAppName] = useState("");
  const [bundleId, setBundleId] = useState<string | undefined>(undefined);
  const [titlePattern, setTitlePattern] = useState("");
  const [commandMode, setCommandMode] = useState<OverrideMode>("merge");
  const [dictionaryMode, setDictionaryMode] = useState<OverrideMode>("merge");
  const [selectedDictionaryIds, setSelectedDictionaryIds] = useState<string[]>([]);
  const [priority, setPriority] = useState(0);
  const [nameError, setNameError] = useState<string | null>(null);
  const [appNameError, setAppNameError] = useState<string | null>(null);
  const [patternError, setPatternError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

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

  const handleAppSelect = useCallback((option: ComboboxOption) => {
    setAppName(option.value);
    setBundleId(option.description);
    setAppNameError(null);
  }, []);

  const validatePattern = (pattern: string): boolean => {
    if (!pattern.trim()) return true;
    try {
      new RegExp(pattern);
      setPatternError(null);
      return true;
    } catch (e) {
      setPatternError(`Invalid regex: ${e instanceof Error ? e.message : String(e)}`);
      return false;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setNameError(null);
    setAppNameError(null);

    if (!name.trim()) {
      setNameError("Name is required");
      return;
    }

    if (!appName.trim()) {
      setAppNameError("App name is required");
      return;
    }

    if (!validatePattern(titlePattern)) {
      return;
    }

    setIsSubmitting(true);
    try {
      await onSubmit({
        name: name.trim(),
        appName: appName.trim(),
        bundleId,
        titlePattern: titlePattern.trim() || undefined,
        commandMode,
        dictionaryMode,
        dictionaryEntryIds: selectedDictionaryIds,
        priority,
        enabled: true,
      });
      setName("");
      setAppName("");
      setBundleId(undefined);
      setTitlePattern("");
      setCommandMode("merge");
      setDictionaryMode("merge");
      setSelectedDictionaryIds([]);
      setPriority(0);
      setPatternError(null);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card>
      <CardContent className="p-4">
        <form onSubmit={handleSubmit}>
          <div className="flex gap-3 items-start flex-wrap">
            <FormField label="Name" error={nameError ?? undefined} className="flex-1 min-w-[150px]">
              <Input
                type="text"
                placeholder="e.g., Slack General"
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  setNameError(null);
                }}
                aria-label="Context name"
              />
            </FormField>
            <FormField label="App Name" error={appNameError ?? undefined} className="flex-1 min-w-[150px]">
              <Combobox
                value={appName}
                onChange={(value) => {
                  setAppName(value);
                  setBundleId(undefined);
                  setAppNameError(null);
                }}
                onSelect={handleAppSelect}
                options={appOptions}
                placeholder="e.g., Slack"
                aria-label="Application name"
              />
            </FormField>
            <FormField label="Title Pattern (regex)" error={patternError ?? undefined} className="flex-1 min-w-[150px]">
              <Input
                type="text"
                placeholder="e.g., .*#general.*"
                value={titlePattern}
                onChange={(e) => {
                  setTitlePattern(e.target.value);
                  validatePattern(e.target.value);
                }}
                aria-label="Window title pattern"
              />
            </FormField>
          </div>
          <div className="flex gap-3 items-center mt-3">
            <FormField label="Command Mode" help="Merge: add to global commands. Replace: use only context commands." className="flex-1">
              <select
                value={commandMode}
                onChange={(e) => setCommandMode(e.target.value as OverrideMode)}
                className="w-full px-3 py-2 rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 text-text-primary"
                aria-label="Command mode"
              >
                <option value="merge">Merge</option>
                <option value="replace">Replace</option>
              </select>
            </FormField>
            <FormField label="Dictionary Mode" help="Merge: add to global dictionary. Replace: use only context dictionary." className="flex-1">
              <select
                value={dictionaryMode}
                onChange={(e) => setDictionaryMode(e.target.value as OverrideMode)}
                className="w-full px-3 py-2 rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 text-text-primary"
                aria-label="Dictionary mode"
              >
                <option value="merge">Merge</option>
                <option value="replace">Replace</option>
              </select>
            </FormField>
            <FormField label="Priority" help="Higher values match first." className="w-24">
              <Input
                type="number"
                value={priority}
                onChange={(e) => setPriority(parseInt(e.target.value, 10) || 0)}
                aria-label="Priority"
              />
            </FormField>
            <div className="pt-6">
              <Button type="submit" disabled={isSubmitting || !!patternError}>
                <Plus className="h-4 w-4" />
                Add
              </Button>
            </div>
          </div>
          {/* Dictionary Entries Section */}
          <FormField label="Dictionary Entries" className="mt-3">
            {dictionaryOptions.length === 0 ? (
              <p className="text-sm text-text-secondary py-2">
                No dictionary entries available. Create entries in the Dictionary page first.
              </p>
            ) : (
              <MultiSelect
                selected={selectedDictionaryIds}
                onChange={setSelectedDictionaryIds}
                options={dictionaryOptions}
                placeholder="Select dictionary entries for this context..."
                aria-label="Dictionary entries"
              />
            )}
          </FormField>
        </form>
      </CardContent>
    </Card>
  );
}

interface ContextItemProps {
  context: WindowContext;
  onEdit: (context: WindowContext) => void;
  onDelete: (id: string) => void;
  onToggleEnabled: (id: string, enabled: boolean) => void;
  isEditing: boolean;
  isDeleting: boolean;
  editValues: {
    name: string;
    appName: string;
    titlePattern: string;
    commandMode: OverrideMode;
    dictionaryMode: OverrideMode;
    dictionaryEntryIds: string[];
    priority: number;
    enabled: boolean;
  };
  editError: string | null;
  patternError: string | null;
  onEditChange: (field: string, value: string | number | boolean | string[]) => void;
  onAppSelect: (option: ComboboxOption) => void;
  appOptions: ComboboxOption[];
  dictionaryOptions: MultiSelectOption[];
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onConfirmDelete: () => void;
  onCancelDelete: () => void;
}

function ContextItem({
  context,
  onEdit,
  onDelete,
  onToggleEnabled,
  isEditing,
  isDeleting,
  editValues,
  editError,
  patternError,
  onEditChange,
  onAppSelect,
  appOptions,
  dictionaryOptions,
  onSaveEdit,
  onCancelEdit,
  onConfirmDelete,
  onCancelDelete,
}: ContextItemProps) {
  if (isEditing) {
    return (
      <Card className="p-3">
        <div className="flex gap-3 items-start flex-wrap">
          <FormField label="Name" error={editError ?? undefined} className="flex-1 min-w-[150px]">
            <Input
              type="text"
              value={editValues.name}
              onChange={(e) => onEditChange("name", e.target.value)}
              aria-label="Edit context name"
            />
          </FormField>
          <FormField label="App Name" className="flex-1 min-w-[150px]">
            <Combobox
              value={editValues.appName}
              onChange={(value) => onEditChange("appName", value)}
              onSelect={onAppSelect}
              options={appOptions}
              aria-label="Edit application name"
            />
          </FormField>
          <FormField label="Title Pattern" error={patternError ?? undefined} className="flex-1 min-w-[150px]">
            <Input
              type="text"
              value={editValues.titlePattern}
              onChange={(e) => onEditChange("titlePattern", e.target.value)}
              aria-label="Edit title pattern"
            />
          </FormField>
        </div>
        <div className="flex gap-3 items-center mt-3">
          <FormField label="Command Mode" help="Merge: add to global commands. Replace: use only context commands." className="flex-1">
            <select
              value={editValues.commandMode}
              onChange={(e) => onEditChange("commandMode", e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 text-text-primary"
            >
              <option value="merge">Merge</option>
              <option value="replace">Replace</option>
            </select>
          </FormField>
          <FormField label="Dictionary Mode" help="Merge: add to global dictionary. Replace: use only context dictionary." className="flex-1">
            <select
              value={editValues.dictionaryMode}
              onChange={(e) => onEditChange("dictionaryMode", e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 text-text-primary"
            >
              <option value="merge">Merge</option>
              <option value="replace">Replace</option>
            </select>
          </FormField>
          <FormField label="Priority" help="Higher values match first." className="w-24">
            <Input
              type="number"
              value={editValues.priority}
              onChange={(e) => onEditChange("priority", parseInt(e.target.value, 10) || 0)}
            />
          </FormField>
          <div className="flex gap-2 pt-6">
            <Button size="sm" onClick={onSaveEdit} disabled={!!patternError} aria-label="Save changes">
              <Check className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="ghost" onClick={onCancelEdit} aria-label="Cancel edit">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {/* Dictionary Entries Section */}
        <FormField label="Dictionary Entries" className="mt-3">
          {dictionaryOptions.length === 0 ? (
            <p className="text-sm text-text-secondary py-2">
              No dictionary entries available.
            </p>
          ) : (
            <MultiSelect
              selected={editValues.dictionaryEntryIds}
              onChange={(ids) => onEditChange("dictionaryEntryIds", ids)}
              options={dictionaryOptions}
              placeholder="Select dictionary entries..."
              aria-label="Dictionary entries"
            />
          )}
        </FormField>
      </Card>
    );
  }

  if (isDeleting) {
    return (
      <Card className="p-3 border-error">
        <div className="flex items-center justify-between">
          <span className="text-text-secondary">Delete "{context.name}"?</span>
          <div className="flex gap-2">
            <Button size="sm" variant="destructive" onClick={onConfirmDelete} aria-label="Confirm delete">
              Confirm
            </Button>
            <Button size="sm" variant="ghost" onClick={onCancelDelete} aria-label="Cancel delete">
              Cancel
            </Button>
          </div>
        </div>
      </Card>
    );
  }

  return (
    <Card className={`p-3 ${!context.enabled ? "opacity-60" : ""}`} role="listitem">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <span className="font-medium text-text-primary shrink-0">{context.name}</span>
          <span className="text-text-secondary text-sm">
            {context.matcher.appName}
            {context.matcher.titlePattern && (
              <span className="text-xs ml-2 text-text-tertiary">/{context.matcher.titlePattern}/</span>
            )}
          </span>
          <div className="flex gap-1">
            <span
              className={`text-xs px-2 py-0.5 rounded ${
                context.commandMode === "replace"
                  ? "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300"
                  : "bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
              }`}
            >
              Cmd: {context.commandMode}
            </span>
            <span
              className={`text-xs px-2 py-0.5 rounded ${
                context.dictionaryMode === "replace"
                  ? "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300"
                  : "bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
              }`}
            >
              Dict: {context.dictionaryMode}
            </span>
            {context.priority !== 0 && (
              <span className="text-xs px-2 py-0.5 rounded bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                P{context.priority}
              </span>
            )}
            {context.dictionaryEntryIds.length > 0 && (
              <span className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300">
                <BookText className="h-3 w-3" />
                {context.dictionaryEntryIds.length}
              </span>
            )}
          </div>
        </div>
        <div className="flex gap-2 shrink-0 items-center">
          <Toggle
            checked={context.enabled}
            onCheckedChange={(enabled) => onToggleEnabled(context.id, enabled)}
            aria-label={`${context.enabled ? "Disable" : "Enable"} ${context.name}`}
          />
          <Button size="sm" variant="ghost" onClick={() => onEdit(context)} aria-label={`Edit ${context.name}`}>
            <Pencil className="h-4 w-4" />
          </Button>
          <Button size="sm" variant="ghost" onClick={() => onDelete(context.id)} aria-label={`Delete ${context.name}`}>
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Card>
  );
}

function EmptyState({ onAddFocus }: { onAddFocus: () => void }) {
  return (
    <Card className="text-center py-12">
      <CardContent className="flex flex-col items-center gap-4">
        <div className="w-16 h-16 rounded-full bg-heycat-orange/10 flex items-center justify-center">
          <Layers className="h-8 w-8 text-heycat-orange" />
        </div>
        <div>
          <h3 className="text-lg font-medium text-text-primary">No window contexts yet</h3>
          <p className="text-sm text-text-secondary mt-1">
            Create contexts to customize commands and dictionary per app
          </p>
        </div>
        <Button onClick={onAddFocus}>
          <Plus className="h-4 w-4" />
          Add Context
        </Button>
      </CardContent>
    </Card>
  );
}

export function WindowContexts(_props: WindowContextsProps) {
  const { toast } = useToast();
  const { contexts, addContext, updateContext, deleteContext } = useWindowContext();
  const runningAppsQuery = useRunningApplications();
  const { entries: dictionaryEntriesQuery } = useDictionary();

  const [searchQuery, setSearchQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({
    name: "",
    appName: "",
    bundleId: undefined as string | undefined,
    titlePattern: "",
    commandMode: "merge" as OverrideMode,
    dictionaryMode: "merge" as OverrideMode,
    dictionaryEntryIds: [] as string[],
    priority: 0,
    enabled: true,
  });
  const [editError, setEditError] = useState<string | null>(null);
  const [patternError, setPatternError] = useState<string | null>(null);

  const contextList = Array.isArray(contexts.data) ? contexts.data : [];
  const runningApps = runningAppsQuery.data ?? [];
  const dictionaryEntries = dictionaryEntriesQuery.data ?? [];

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

  const filteredContexts = useMemo(() => {
    if (!searchQuery.trim()) return contextList;
    const query = searchQuery.toLowerCase();
    return contextList.filter(
      (ctx) =>
        ctx.name.toLowerCase().includes(query) ||
        ctx.matcher.appName.toLowerCase().includes(query) ||
        (ctx.matcher.titlePattern?.toLowerCase().includes(query) ?? false)
    );
  }, [contextList, searchQuery]);

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

  const validatePattern = (pattern: string): boolean => {
    if (!pattern.trim()) {
      setPatternError(null);
      return true;
    }
    try {
      new RegExp(pattern);
      setPatternError(null);
      return true;
    } catch (e) {
      setPatternError(`Invalid regex: ${e instanceof Error ? e.message : String(e)}`);
      return false;
    }
  };

  const handleEditChange = useCallback((field: string, value: string | number | boolean | string[]) => {
    setEditValues((prev) => {
      // Clear bundleId when appName is manually typed
      if (field === "appName") {
        return { ...prev, [field]: value, bundleId: undefined };
      }
      return { ...prev, [field]: value };
    });
    if (field === "name") setEditError(null);
    if (field === "titlePattern" && typeof value === "string") {
      validatePattern(value);
    }
  }, []);

  const handleEditAppSelect = useCallback((option: ComboboxOption) => {
    setEditValues((prev) => ({
      ...prev,
      appName: option.value,
      bundleId: option.description,
    }));
  }, []);

  const handleSaveEdit = useCallback(async () => {
    if (!editingId) return;

    if (!editValues.name.trim()) {
      setEditError("Name is required");
      return;
    }

    if (!editValues.appName.trim()) {
      setEditError("App name is required");
      return;
    }

    if (!validatePattern(editValues.titlePattern)) {
      return;
    }

    try {
      await updateContext.mutateAsync({
        id: editingId,
        name: editValues.name.trim(),
        appName: editValues.appName.trim(),
        bundleId: editValues.bundleId,
        titlePattern: editValues.titlePattern.trim() || undefined,
        commandMode: editValues.commandMode,
        dictionaryMode: editValues.dictionaryMode,
        dictionaryEntryIds: editValues.dictionaryEntryIds,
        priority: editValues.priority,
        enabled: editValues.enabled,
      });
      toast({
        type: "success",
        title: "Context updated",
        description: `"${editValues.name}" has been updated.`,
      });
      setEditingId(null);
      setEditValues({
        name: "",
        appName: "",
        bundleId: undefined,
        titlePattern: "",
        commandMode: "merge",
        dictionaryMode: "merge",
        dictionaryEntryIds: [],
        priority: 0,
        enabled: true,
      });
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
    setEditValues({
      name: "",
      appName: "",
      bundleId: undefined,
      titlePattern: "",
      commandMode: "merge",
      dictionaryMode: "merge",
      dictionaryEntryIds: [],
      priority: 0,
      enabled: true,
    });
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
          titlePattern: ctx.matcher.titlePattern,
          commandMode: ctx.commandMode,
          dictionaryMode: ctx.dictionaryMode,
          priority: ctx.priority,
          enabled,
        });
        toast({
          type: "success",
          title: enabled ? "Context enabled" : "Context disabled",
          description: `"${ctx.name}" has been ${enabled ? "enabled" : "disabled"}.`,
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

  const handleConfirmDelete = useCallback(async () => {
    if (!deleteConfirmId) return;

    const ctx = contextList.find((c) => c.id === deleteConfirmId);
    try {
      await deleteContext.mutateAsync({ id: deleteConfirmId });
      toast({
        type: "success",
        title: "Context deleted",
        description: ctx ? `"${ctx.name}" has been removed.` : "Context removed.",
      });
      setDeleteConfirmId(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to delete context",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }, [deleteConfirmId, contextList, deleteContext, toast]);

  const handleFocusAdd = useCallback(() => {
    const nameInput = document.querySelector('input[aria-label="Context name"]') as HTMLInputElement | null;
    nameInput?.focus();
  }, []);

  if (contexts.isLoading) {
    return (
      <div className="p-6">
        <div className="text-text-secondary" role="status">
          Loading window contexts...
        </div>
      </div>
    );
  }

  if (contexts.isError) {
    return (
      <div className="p-6">
        <Card className="border-error">
          <CardContent>
            <div className="text-error" role="alert">
              {contexts.error instanceof Error ? contexts.error.message : "Failed to load window contexts"}
            </div>
            <Button onClick={() => contexts.refetch()} className="mt-4">
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">Window Contexts</h1>
        <p className="text-text-secondary mt-1">
          Create app-specific configurations for commands and dictionary.
        </p>
      </header>

      {/* Add Context Form */}
      <AddContextForm onSubmit={handleAddContext} runningApps={runningApps} dictionaryEntries={dictionaryEntries} />

      {/* Search Bar */}
      {contextList.length > 0 && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <Input
            type="text"
            placeholder="Search contexts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
            aria-label="Search window contexts"
          />
        </div>
      )}

      {/* Context List or Empty State */}
      {contextList.length === 0 ? (
        <EmptyState onAddFocus={handleFocusAdd} />
      ) : filteredContexts.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">No contexts match "{searchQuery}"</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2" role="list" aria-label="Window contexts">
          {filteredContexts.map((ctx) => (
            <ContextItem
              key={ctx.id}
              context={ctx}
              onEdit={handleStartEdit}
              onDelete={(id) => setDeleteConfirmId(id)}
              onToggleEnabled={handleToggleEnabled}
              isEditing={editingId === ctx.id}
              isDeleting={deleteConfirmId === ctx.id}
              editValues={editValues}
              editError={editError}
              patternError={patternError}
              onEditChange={handleEditChange}
              onAppSelect={handleEditAppSelect}
              appOptions={appOptions}
              dictionaryOptions={dictionaryOptions}
              onSaveEdit={handleSaveEdit}
              onCancelEdit={handleCancelEdit}
              onConfirmDelete={handleConfirmDelete}
              onCancelDelete={() => setDeleteConfirmId(null)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
