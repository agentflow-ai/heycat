import { useState } from "react";
import { Pencil, Trash2, Check, X, Settings, Layers } from "lucide-react";
import { Card, Button, Input, FormField, MultiSelect } from "../../components/ui";
import type { DictionaryEntry } from "../../types/dictionary";
import type { WindowContext } from "../../types/windowContext";
import { EntrySettings } from "./EntrySettings";
import { useDictionaryContext } from "./DictionaryContext";

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

export interface EntryItemProps {
  entry: DictionaryEntry;
}

/**
 * Single dictionary entry item.
 * Uses DictionaryContext for edit state and actions.
 */
export function EntryItem({ entry }: EntryItemProps) {
  const {
    contextsByEntryId,
    contextOptions,
    editingId,
    editValues,
    editError,
    editSuffixError,
    deletion,
    handleStartEdit,
    handleEditChange,
    handleSaveEdit,
    handleCancelEdit,
  } = useDictionaryContext();

  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const assignedContexts = contextsByEntryId.get(entry.id) ?? [];
  const isEditing = editingId === entry.id;
  const isDeleting = deletion.isConfirming(entry.id);

  const hasSettings = entry.suffix || entry.autoEnter || entry.disableSuffix || entry.completeMatchOnly;
  const editHasSettings =
    editValues.suffix !== "" || editValues.autoEnter || editValues.disableSuffix || editValues.completeMatchOnly;

  if (isEditing) {
    return (
      <Card className="p-3">
        <div className="flex gap-3 items-start">
          <FormField label="Trigger" error={editError ?? undefined} className="flex-1">
            <Input
              type="text"
              value={editValues.trigger}
              onChange={(e) => handleEditChange("trigger", e.target.value)}
              aria-label="Edit trigger phrase"
              aria-invalid={!!editError}
            />
          </FormField>
          <FormField label="Expansion" className="flex-[2]">
            <Input
              type="text"
              value={editValues.expansion}
              onChange={(e) => handleEditChange("expansion", e.target.value)}
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
              onClick={handleSaveEdit}
              disabled={!!editSuffixError}
              aria-label="Save changes"
            >
              <Check className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="ghost" onClick={handleCancelEdit} aria-label="Cancel edit">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {isSettingsOpen && (
          <>
            <EntrySettings
              suffix={editValues.suffix}
              autoEnter={editValues.autoEnter}
              disableSuffix={editValues.disableSuffix}
              completeMatchOnly={editValues.completeMatchOnly}
              onSuffixChange={(value) => handleEditChange("suffix", value)}
              onAutoEnterChange={(value) => handleEditChange("autoEnter", value)}
              onDisableSuffixChange={(value) => handleEditChange("disableSuffix", value)}
              onCompleteMatchOnlyChange={(value) => handleEditChange("completeMatchOnly", value)}
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
                  onChange={(ids) => handleEditChange("contextIds", ids)}
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
          <span className="text-text-secondary">Delete "{entry.trigger}"?</span>
          <div className="flex gap-2">
            <Button size="sm" variant="danger" onClick={deletion.confirmDelete} aria-label="Confirm delete">
              Confirm
            </Button>
            <Button size="sm" variant="ghost" onClick={deletion.cancelDelete} aria-label="Cancel delete">
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
          <span className="font-medium text-text-primary shrink-0">"{entry.trigger}"</span>
          <span className="text-text-secondary">→</span>
          <span className="text-text-secondary truncate">{entry.expansion}</span>
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
            onClick={() => handleStartEdit(entry)}
            aria-label={`Edit ${entry.trigger}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => deletion.requestDelete(entry.id)}
            aria-label={`Delete ${entry.trigger}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Card>
  );
}
