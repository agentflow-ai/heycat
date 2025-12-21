import { useState, useMemo, useCallback } from "react";
import { Plus, Search, Book, Pencil, Trash2, Check, X } from "lucide-react";
import { Card, CardContent, Button, Input, FormField } from "../components/ui";
import { useToast } from "../components/overlays";
import { useDictionary } from "../hooks/useDictionary";
import type { DictionaryEntry } from "../types/dictionary";

export interface DictionaryProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

interface AddEntryFormProps {
  onSubmit: (trigger: string, expansion: string) => Promise<void>;
  existingTriggers: string[];
}

function AddEntryForm({ onSubmit, existingTriggers }: AddEntryFormProps) {
  const [trigger, setTrigger] = useState("");
  const [expansion, setExpansion] = useState("");
  const [triggerError, setTriggerError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

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

    setIsSubmitting(true);
    try {
      await onSubmit(trigger.trim(), expansion.trim());
      setTrigger("");
      setExpansion("");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card>
      <CardContent className="p-4">
        <form onSubmit={handleSubmit} className="flex gap-3 items-start">
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
          <div className="pt-6">
            <Button type="submit" disabled={isSubmitting}>
              <Plus className="h-4 w-4" />
              Add
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

interface EntryItemProps {
  entry: DictionaryEntry;
  onEdit: (entry: DictionaryEntry) => void;
  onDelete: (id: string) => void;
  isEditing: boolean;
  isDeleting: boolean;
  editValues: { trigger: string; expansion: string };
  editError: string | null;
  onEditChange: (field: "trigger" | "expansion", value: string) => void;
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onConfirmDelete: () => void;
  onCancelDelete: () => void;
}

function EntryItem({
  entry,
  onEdit,
  onDelete,
  isEditing,
  isDeleting,
  editValues,
  editError,
  onEditChange,
  onSaveEdit,
  onCancelEdit,
  onConfirmDelete,
  onCancelDelete,
}: EntryItemProps) {
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
              onClick={onSaveEdit}
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
              variant="destructive"
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

  const [searchQuery, setSearchQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({ trigger: "", expansion: "" });
  const [editError, setEditError] = useState<string | null>(null);

  const entryList = Array.isArray(entries.data) ? entries.data : [];

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
    async (trigger: string, expansion: string) => {
      try {
        await addEntry.mutateAsync({ trigger, expansion });
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
    [addEntry, toast]
  );

  const handleStartEdit = useCallback((entry: DictionaryEntry) => {
    setEditingId(entry.id);
    setEditValues({ trigger: entry.trigger, expansion: entry.expansion });
    setEditError(null);
  }, []);

  const handleEditChange = useCallback(
    (field: "trigger" | "expansion", value: string) => {
      setEditValues((prev) => ({ ...prev, [field]: value }));
      if (field === "trigger") {
        setEditError(null);
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

    try {
      await updateEntry.mutateAsync({
        id: editingId,
        trigger: trimmedTrigger,
        expansion: editValues.expansion.trim(),
      });
      toast({
        type: "success",
        title: "Entry updated",
        description: `"${trimmedTrigger}" has been updated.`,
      });
      setEditingId(null);
      setEditValues({ trigger: "", expansion: "" });
      setEditError(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to update entry",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  }, [editingId, editValues, entryList, updateEntry, toast]);

  const handleCancelEdit = useCallback(() => {
    setEditingId(null);
    setEditValues({ trigger: "", expansion: "" });
    setEditError(null);
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
              onEdit={handleStartEdit}
              onDelete={(id) => setDeleteConfirmId(id)}
              isEditing={editingId === entry.id}
              isDeleting={deleteConfirmId === entry.id}
              editValues={editValues}
              editError={editError}
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
