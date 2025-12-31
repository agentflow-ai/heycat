import { useState, useMemo, useCallback } from "react";
import { Plus, Search, Book, Pencil, Trash2, Check, X, Settings, Layers } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, Toggle, MultiSelect } from "../components/ui";
import type { MultiSelectOption } from "../components/ui";
import { useToast } from "../components/overlays";
import { useDictionary } from "../hooks/useDictionary";
import { useWindowContext } from "../hooks/useWindowContext";
import type { DictionaryEntry } from "../types/dictionary";
import type { WindowContext } from "../types/windowContext";

export interface DictionaryProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

interface SettingsPanelProps {
  suffix: string;
  autoEnter: boolean;
  disableSuffix: boolean;
  completeMatchOnly: boolean;
  onSuffixChange: (value: string) => void;
  onAutoEnterChange: (value: boolean) => void;
  onDisableSuffixChange: (value: boolean) => void;
  onCompleteMatchOnlyChange: (value: boolean) => void;
  suffixError?: string | null;
}

function SettingsPanel({
  suffix,
  autoEnter,
  disableSuffix,
  completeMatchOnly,
  onSuffixChange,
  onAutoEnterChange,
  onDisableSuffixChange,
  onCompleteMatchOnlyChange,
  suffixError,
}: SettingsPanelProps) {
  return (
    <div
      className="mt-3 p-3 bg-neutral-100 dark:bg-neutral-800 rounded-lg border border-neutral-200 dark:border-neutral-700 animate-slideDown"
      data-testid="settings-panel"
    >
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between gap-4">
          <label
            htmlFor="suffix-input"
            className="text-sm font-medium text-text-secondary"
          >
            Suffix
          </label>
          <div className="flex flex-col items-end gap-1">
            <Input
              id="suffix-input"
              type="text"
              value={suffix}
              onChange={(e) => onSuffixChange(e.target.value)}
              placeholder="e.g., . or ?"
              maxLength={5}
              disabled={disableSuffix}
              className={`w-20 text-center ${suffixError ? "border-error" : ""} ${disableSuffix ? "opacity-50" : ""}`}
              aria-label="Suffix"
              aria-invalid={!!suffixError}
            />
            {suffixError && (
              <span className="text-error text-xs">{suffixError}</span>
            )}
          </div>
        </div>
        <div className="flex items-center justify-between gap-4">
          <label
            htmlFor="disable-suffix-toggle"
            className="text-sm font-medium text-text-secondary"
            title="When enabled, no punctuation will be added after expansion"
          >
            No punctuation
          </label>
          <Toggle
            id="disable-suffix-toggle"
            checked={disableSuffix}
            onCheckedChange={onDisableSuffixChange}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <label
            htmlFor="auto-enter-toggle"
            className="text-sm font-medium text-text-secondary"
          >
            Auto-enter
          </label>
          <Toggle
            id="auto-enter-toggle"
            checked={autoEnter}
            onCheckedChange={onAutoEnterChange}
          />
        </div>
        <div className="flex items-center justify-between gap-4">
          <label
            htmlFor="complete-match-only-toggle"
            className="text-sm font-medium text-text-secondary"
            title="When enabled, trigger only expands if it's the entire transcription input"
          >
            Complete match only
          </label>
          <Toggle
            id="complete-match-only-toggle"
            checked={completeMatchOnly}
            onCheckedChange={onCompleteMatchOnlyChange}
          />
        </div>
      </div>
    </div>
  );
}

interface ContextBadgesProps {
  contexts: WindowContext[];
}

function ContextBadges({ contexts }: ContextBadgesProps) {
  if (contexts.length === 0) {
    return (
      <span
        className="text-xs px-2 py-0.5 rounded bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
        data-testid="context-badge-global"
      >
        Global
      </span>
    );
  }

  if (contexts.length === 1) {
    return (
      <span
        className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
        data-testid="context-badge"
      >
        <Layers className="h-3 w-3" />
        {contexts[0].name}
      </span>
    );
  }

  if (contexts.length === 2) {
    return (
      <div className="flex gap-1">
        {contexts.map((ctx) => (
          <span
            key={ctx.id}
            className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
            data-testid="context-badge"
          >
            <Layers className="h-3 w-3" />
            {ctx.name}
          </span>
        ))}
      </div>
    );
  }

  // 3+ contexts: show count with tooltip
  const contextNames = contexts.map((c) => c.name).join(", ");
  return (
    <span
      className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300 cursor-help"
      title={contextNames}
      data-testid="context-badge-count"
    >
      <Layers className="h-3 w-3" />
      {contexts.length} contexts
    </span>
  );
}

interface AddEntryFormProps {
  onSubmit: (
    trigger: string,
    expansion: string,
    contextIds: string[],
    suffix?: string,
    autoEnter?: boolean,
    disableSuffix?: boolean,
    completeMatchOnly?: boolean
  ) => Promise<void>;
  existingTriggers: string[];
  contextOptions: MultiSelectOption[];
}

function AddEntryForm({ onSubmit, existingTriggers, contextOptions }: AddEntryFormProps) {
  const [trigger, setTrigger] = useState("");
  const [expansion, setExpansion] = useState("");
  const [suffix, setSuffix] = useState("");
  const [autoEnter, setAutoEnter] = useState(false);
  const [disableSuffix, setDisableSuffix] = useState(false);
  const [completeMatchOnly, setCompleteMatchOnly] = useState(false);
  const [selectedContextIds, setSelectedContextIds] = useState<string[]>([]);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [triggerError, setTriggerError] = useState<string | null>(null);
  const [suffixError, setSuffixError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const hasSettings = suffix !== "" || autoEnter || disableSuffix || completeMatchOnly;

  const validateSuffix = (value: string): boolean => {
    if (value.length > 5) {
      setSuffixError("Suffix must be 5 characters or less");
      return false;
    }
    setSuffixError(null);
    return true;
  };

  const handleSuffixChange = (value: string) => {
    setSuffix(value);
    validateSuffix(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setTriggerError(null);

    // Validate trigger
    if (!trigger.trim()) {
      setTriggerError("Trigger is required");
      return;
    }

    if (existingTriggers.includes(trigger.toLowerCase())) {
      setTriggerError("This trigger already exists");
      return;
    }

    // Validate suffix
    if (!validateSuffix(suffix)) {
      return;
    }

    setIsSubmitting(true);
    try {
      await onSubmit(
        trigger.trim(),
        expansion.trim(),
        selectedContextIds,
        disableSuffix ? undefined : (suffix.trim() || undefined),
        autoEnter || undefined,
        disableSuffix || undefined,
        completeMatchOnly || undefined
      );
      setTrigger("");
      setExpansion("");
      setSuffix("");
      setSuffixError(null);
      setAutoEnter(false);
      setDisableSuffix(false);
      setCompleteMatchOnly(false);
      setSelectedContextIds([]);
      setIsSettingsOpen(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card>
      <CardContent className="p-4">
        <form onSubmit={handleSubmit}>
          <div className="flex gap-3 items-start">
            <FormField
              label="Trigger"
              error={triggerError ?? undefined}
              className="flex-1"
            >
              <Input
                type="text"
                placeholder="e.g., brb"
                value={trigger}
                onChange={(e) => {
                  setTrigger(e.target.value);
                  setTriggerError(null);
                }}
                aria-label="Trigger phrase"
                aria-invalid={!!triggerError}
              />
            </FormField>
            <FormField label="Expansion" className="flex-[2]">
              <Input
                type="text"
                placeholder="e.g., be right back"
                value={expansion}
                onChange={(e) => setExpansion(e.target.value)}
                aria-label="Expansion text"
              />
            </FormField>
            <div className="pt-6 flex gap-2">
              <Button
                type="button"
                variant="ghost"
                onClick={() => setIsSettingsOpen(!isSettingsOpen)}
                aria-label="Toggle settings"
                aria-expanded={isSettingsOpen}
                className={hasSettings ? "text-heycat-orange" : ""}
              >
                <Settings className="h-4 w-4" />
                {hasSettings && (
                  <span className="absolute top-1 right-1 w-2 h-2 bg-heycat-orange rounded-full" />
                )}
              </Button>
              <Button type="submit" disabled={isSubmitting || !!suffixError}>
                <Plus className="h-4 w-4" />
                Add
              </Button>
            </div>
          </div>
          {isSettingsOpen && (
            <>
              <SettingsPanel
                suffix={suffix}
                autoEnter={autoEnter}
                disableSuffix={disableSuffix}
                completeMatchOnly={completeMatchOnly}
                onSuffixChange={handleSuffixChange}
                onAutoEnterChange={setAutoEnter}
                onDisableSuffixChange={setDisableSuffix}
                onCompleteMatchOnlyChange={setCompleteMatchOnly}
                suffixError={suffixError}
              />
              {contextOptions.length > 0 && (
                <FormField
                  label="Window Contexts"
                  help="Assign this entry to specific app contexts. Leave empty for global availability."
                  className="mt-3"
                >
                  <MultiSelect
                    selected={selectedContextIds}
                    onChange={setSelectedContextIds}
                    options={contextOptions}
                    placeholder="Select contexts (optional)..."
                    aria-label="Window contexts"
                  />
                </FormField>
              )}
            </>
          )}
        </form>
      </CardContent>
    </Card>
  );
}

interface EntryItemProps {
  entry: DictionaryEntry;
  assignedContexts: WindowContext[];
  onEdit: (entry: DictionaryEntry) => void;
  onDelete: (id: string) => void;
  isEditing: boolean;
  isDeleting: boolean;
  editValues: { trigger: string; expansion: string; suffix: string; autoEnter: boolean; disableSuffix: boolean; completeMatchOnly: boolean; contextIds: string[] };
  editError: string | null;
  editSuffixError: string | null;
  contextOptions: MultiSelectOption[];
  onEditChange: (field: "trigger" | "expansion" | "suffix" | "autoEnter" | "disableSuffix" | "completeMatchOnly" | "contextIds", value: string | boolean | string[]) => void;
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onConfirmDelete: () => void;
  onCancelDelete: () => void;
}

function EntryItem({
  entry,
  assignedContexts,
  onEdit,
  onDelete,
  isEditing,
  isDeleting,
  editValues,
  editError,
  editSuffixError,
  contextOptions,
  onEditChange,
  onSaveEdit,
  onCancelEdit,
  onConfirmDelete,
  onCancelDelete,
}: EntryItemProps) {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const hasSettings = entry.suffix || entry.autoEnter || entry.disableSuffix || entry.completeMatchOnly;
  const editHasSettings = editValues.suffix !== "" || editValues.autoEnter || editValues.disableSuffix || editValues.completeMatchOnly;

  if (isEditing) {
    return (
      <Card className="p-3">
        <div className="flex gap-3 items-start">
          <FormField
            label="Trigger"
            error={editError ?? undefined}
            className="flex-1"
          >
            <Input
              type="text"
              value={editValues.trigger}
              onChange={(e) => onEditChange("trigger", e.target.value)}
              aria-label="Edit trigger phrase"
              aria-invalid={!!editError}
            />
          </FormField>
          <FormField label="Expansion" className="flex-[2]">
            <Input
              type="text"
              value={editValues.expansion}
              onChange={(e) => onEditChange("expansion", e.target.value)}
              aria-label="Edit expansion text"
            />
          </FormField>
          <div className="flex gap-2 pt-6">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setIsSettingsOpen(!isSettingsOpen)}
              aria-label="Toggle settings"
              aria-expanded={isSettingsOpen}
              className={editHasSettings ? "text-heycat-orange" : ""}
            >
              <Settings className="h-4 w-4" />
            </Button>
            <Button
              size="sm"
              onClick={onSaveEdit}
              disabled={!!editSuffixError}
              aria-label="Save changes"
            >
              <Check className="h-4 w-4" />
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={onCancelEdit}
              aria-label="Cancel edit"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {isSettingsOpen && (
          <>
            <SettingsPanel
              suffix={editValues.suffix}
              autoEnter={editValues.autoEnter}
              disableSuffix={editValues.disableSuffix}
              completeMatchOnly={editValues.completeMatchOnly}
              onSuffixChange={(value) => onEditChange("suffix", value)}
              onAutoEnterChange={(value) => onEditChange("autoEnter", value)}
              onDisableSuffixChange={(value) => onEditChange("disableSuffix", value)}
              onCompleteMatchOnlyChange={(value) => onEditChange("completeMatchOnly", value)}
              suffixError={editSuffixError}
            />
            {contextOptions.length > 0 && (
              <FormField
                label="Window Contexts"
                help="Assign this entry to specific app contexts. Leave empty for global availability."
                className="mt-3"
              >
                <MultiSelect
                  selected={editValues.contextIds}
                  onChange={(ids) => onEditChange("contextIds", ids)}
                  options={contextOptions}
                  placeholder="Select contexts (optional)..."
                  aria-label="Window contexts"
                />
              </FormField>
            )}
          </>
        )}
      </Card>
    );
  }

  if (isDeleting) {
    return (
      <Card className="p-3 border-error">
        <div className="flex items-center justify-between">
          <span className="text-text-secondary">
            Delete "{entry.trigger}"?
          </span>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="danger"
              onClick={onConfirmDelete}
              aria-label="Confirm delete"
            >
              Confirm
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={onCancelDelete}
              aria-label="Cancel delete"
            >
              Cancel
            </Button>
          </div>
        </div>
      </Card>
    );
  }

  return (
    <Card className="p-3" role="listitem">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <span className="font-medium text-text-primary shrink-0">
            "{entry.trigger}"
          </span>
          <span className="text-text-secondary">→</span>
          <span className="text-text-secondary truncate">
            {entry.expansion}
          </span>
          {hasSettings && (
            <span
              className="text-heycat-orange shrink-0"
              title={`${entry.disableSuffix ? "No punctuation" : `Suffix: "${entry.suffix || ""}"`}${entry.autoEnter ? ", Auto-enter" : ""}${entry.completeMatchOnly ? ", Complete match only" : ""}`}
            >
              <Settings className="h-4 w-4" />
            </span>
          )}
          <ContextBadges contexts={assignedContexts} />
        </div>
        <div className="flex gap-2 shrink-0">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onEdit(entry)}
            aria-label={`Edit ${entry.trigger}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onDelete(entry.id)}
            aria-label={`Delete ${entry.trigger}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Card>
  );
}

function DictionaryEmptyState({ onAddFocus }: { onAddFocus: () => void }) {
  return (
    <Card className="text-center py-12">
      <CardContent className="flex flex-col items-center gap-4">
        <div className="w-16 h-16 rounded-full bg-heycat-orange/10 flex items-center justify-center">
          <Book className="h-8 w-8 text-heycat-orange" />
        </div>
        <div>
          <h3 className="text-lg font-medium text-text-primary">
            No dictionary entries yet
          </h3>
          <p className="text-sm text-text-secondary mt-1">
            Add your first text expansion to get started
          </p>
        </div>
        <Button onClick={onAddFocus}>
          <Plus className="h-4 w-4" />
          Add Entry
        </Button>
      </CardContent>
    </Card>
  );
}

export function Dictionary(_props: DictionaryProps) {
  const { toast } = useToast();
  const { entries, addEntry, updateEntry, deleteEntry } = useDictionary();
  const { contexts, updateContext } = useWindowContext();

  const [searchQuery, setSearchQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({
    trigger: "",
    expansion: "",
    suffix: "",
    autoEnter: false,
    disableSuffix: false,
    completeMatchOnly: false,
    contextIds: [] as string[],
  });
  const [editError, setEditError] = useState<string | null>(null);
  const [editSuffixError, setEditSuffixError] = useState<string | null>(null);

  const entryList = Array.isArray(entries.data) ? entries.data : [];
  const contextList = Array.isArray(contexts.data) ? contexts.data : [];

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

  const filteredEntries = useMemo(() => {
    if (!searchQuery.trim()) return entryList;
    const query = searchQuery.toLowerCase();
    return entryList.filter(
      (entry) =>
        entry.trigger.toLowerCase().includes(query) ||
        entry.expansion.toLowerCase().includes(query)
    );
  }, [entryList, searchQuery]);

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
        const newEntry = await addEntry.mutateAsync({ trigger, expansion, suffix, autoEnter, disableSuffix, completeMatchOnly });

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

  const handleStartEdit = useCallback((entry: DictionaryEntry) => {
    setEditingId(entry.id);
    // Get current context assignments for this entry
    const assignedContextIds = (contextsByEntryId.get(entry.id) ?? []).map(ctx => ctx.id);
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
  }, [contextsByEntryId]);

  const handleEditChange = useCallback(
    (field: "trigger" | "expansion" | "suffix" | "autoEnter" | "disableSuffix" | "completeMatchOnly" | "contextIds", value: string | boolean | string[]) => {
      setEditValues((prev) => ({ ...prev, [field]: value }));
      if (field === "trigger") {
        setEditError(null);
      }
      if (field === "suffix" && typeof value === "string") {
        if (value.length > 5) {
          setEditSuffixError("Suffix must be 5 characters or less");
        } else {
          setEditSuffixError(null);
        }
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
      (e) =>
        e.id !== editingId &&
        e.trigger.toLowerCase() === trimmedTrigger.toLowerCase()
    );
    if (isDuplicate) {
      setEditError("This trigger already exists");
      return;
    }

    // Validate suffix
    if (editValues.suffix.length > 5) {
      setEditSuffixError("Suffix must be 5 characters or less");
      return;
    }

    try {
      await updateEntry.mutateAsync({
        id: editingId,
        trigger: trimmedTrigger,
        expansion: editValues.expansion.trim(),
        suffix: editValues.disableSuffix ? undefined : (editValues.suffix.trim() || undefined),
        autoEnter: editValues.autoEnter || undefined,
        disableSuffix: editValues.disableSuffix || undefined,
        completeMatchOnly: editValues.completeMatchOnly || undefined,
      });

      // Sync context assignments
      const oldContextIds = (contextsByEntryId.get(editingId) ?? []).map(ctx => ctx.id);
      const newContextIds = editValues.contextIds;

      // Contexts to add entry to
      const contextsToAdd = newContextIds.filter(id => !oldContextIds.includes(id));
      // Contexts to remove entry from
      const contextsToRemove = oldContextIds.filter(id => !newContextIds.includes(id));

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
            dictionaryEntryIds: ctx.dictionaryEntryIds.filter(id => id !== editingId),
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
      setEditValues({ trigger: "", expansion: "", suffix: "", autoEnter: false, disableSuffix: false, completeMatchOnly: false, contextIds: [] });
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
    setEditValues({ trigger: "", expansion: "", suffix: "", autoEnter: false, disableSuffix: false, completeMatchOnly: false, contextIds: [] });
    setEditError(null);
    setEditSuffixError(null);
  }, []);

  const handleConfirmDelete = useCallback(async () => {
    if (!deleteConfirmId) return;

    const entry = entryList.find((e) => e.id === deleteConfirmId);
    try {
      await deleteEntry.mutateAsync({ id: deleteConfirmId });
      toast({
        type: "success",
        title: "Entry deleted",
        description: entry
          ? `"${entry.trigger}" has been removed.`
          : "Entry removed.",
      });
      setDeleteConfirmId(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to delete entry",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }, [deleteConfirmId, entryList, deleteEntry, toast]);

  const handleFocusAdd = useCallback(() => {
    // Focus the trigger input in the add form
    const triggerInput = document.querySelector(
      'input[aria-label="Trigger phrase"]'
    ) as HTMLInputElement | null;
    triggerInput?.focus();
  }, []);

  if (entries.isLoading) {
    return (
      <div className="p-6">
        <div className="text-text-secondary" role="status">
          Loading dictionary...
        </div>
      </div>
    );
  }

  if (entries.isError) {
    return (
      <div className="p-6">
        <Card className="border-error">
          <CardContent>
            <div className="text-error" role="alert">
              {entries.error instanceof Error
                ? entries.error.message
                : "Failed to load dictionary"}
            </div>
            <Button onClick={() => entries.refetch()} className="mt-4">
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
        <h1 className="text-2xl font-semibold text-text-primary">Dictionary</h1>
        <p className="text-text-secondary mt-1">
          Create text expansions to speed up your typing.
        </p>
      </header>

      {/* Add Entry Form */}
      <AddEntryForm
        onSubmit={handleAddEntry}
        existingTriggers={existingTriggers.filter(
          (t) => t !== editValues.trigger.toLowerCase()
        )}
        contextOptions={contextOptions}
      />

      {/* Search Bar */}
      {entryList.length > 0 && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <Input
            type="text"
            placeholder="Search entries..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
            aria-label="Search dictionary entries"
          />
        </div>
      )}

      {/* Entry List or Empty State */}
      {entryList.length === 0 ? (
        <DictionaryEmptyState onAddFocus={handleFocusAdd} />
      ) : filteredEntries.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">
              No entries match "{searchQuery}"
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2" role="list" aria-label="Dictionary entries">
          {filteredEntries.map((entry) => (
            <EntryItem
              key={entry.id}
              entry={entry}
              assignedContexts={contextsByEntryId.get(entry.id) ?? []}
              onEdit={handleStartEdit}
              onDelete={(id) => setDeleteConfirmId(id)}
              isEditing={editingId === entry.id}
              isDeleting={deleteConfirmId === entry.id}
              editValues={editValues}
              editError={editError}
              editSuffixError={editSuffixError}
              contextOptions={contextOptions}
              onEditChange={handleEditChange}
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
