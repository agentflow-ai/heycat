import { useState } from "react";
import { Pencil, Trash2, Check, X, BookText } from "lucide-react";
import { Card, Button, Input, FormField, Toggle, Combobox, MultiSelect } from "../../components/ui";
import type { ComboboxOption } from "../../components/ui";
import type { WindowContext, OverrideMode } from "../../types/windowContext";
import { validateRegexPattern } from "../../lib/validation";
import { useWindowContextsContext } from "./WindowContextsContext";

export interface ContextItemProps {
  context: WindowContext;
}

/**
 * Single window context item.
 * Uses WindowContextsContext for edit state and actions.
 */
export function ContextItem({ context }: ContextItemProps) {
  const {
    appOptions,
    dictionaryOptions,
    editingId,
    editValues,
    editError,
    patternError,
    deletion,
    handleStartEdit,
    handleEditChange,
    handleSaveEdit,
    handleCancelEdit,
    handleToggleEnabled,
  } = useWindowContextsContext();

  const [localPatternError, setLocalPatternError] = useState<string | null>(null);

  const isEditing = editingId === context.id;
  const isDeleting = deletion.isConfirming(context.id);

  const handleAppSelect = (option: ComboboxOption) => {
    handleEditChange("appName", option.value);
    handleEditChange("bundleId", option.description);
  };

  const handlePatternChange = (value: string) => {
    handleEditChange("titlePattern", value);
    const error = validateRegexPattern(value);
    setLocalPatternError(error);
  };

  if (isEditing) {
    return (
      <Card className="p-3">
        <div className="flex gap-3 items-start flex-wrap">
          <FormField label="Name" error={editError ?? undefined} className="flex-1 min-w-[150px]">
            <Input
              type="text"
              value={editValues.name}
              onChange={(e) => handleEditChange("name", e.target.value)}
              aria-label="Edit context name"
            />
          </FormField>
          <FormField label="App Name" className="flex-1 min-w-[150px]">
            <Combobox
              value={editValues.appName}
              onChange={(value) => {
                handleEditChange("appName", value);
                handleEditChange("bundleId", undefined);
              }}
              onSelect={handleAppSelect}
              options={appOptions}
              placeholder="Select or type app name..."
              aria-label="Edit app name"
            />
          </FormField>
          <FormField
            label="Title Pattern"
            error={localPatternError ?? patternError ?? undefined}
            className="flex-1 min-w-[150px]"
          >
            <Input
              type="text"
              value={editValues.titlePattern}
              onChange={(e) => handlePatternChange(e.target.value)}
              placeholder="e.g., #general.*"
              aria-label="Edit window title pattern"
            />
          </FormField>
          <div className="flex gap-2 pt-6">
            <Button
              size="sm"
              onClick={handleSaveEdit}
              disabled={!!localPatternError || !!patternError}
              aria-label="Save changes"
            >
              <Check className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="ghost" onClick={handleCancelEdit} aria-label="Cancel edit">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
        <div className="mt-3 flex gap-4 items-center">
          <div className="flex items-center gap-2">
            <label className="text-sm text-text-secondary">Command mode:</label>
            <Toggle
              checked={editValues.commandMode === "override"}
              onCheckedChange={(checked) =>
                handleEditChange("commandMode", checked ? "override" : "merge")
              }
              aria-label="Command mode context only"
            />
            <span className="text-xs text-text-tertiary">
              {editValues.commandMode === "override" ? "Context Only" : "Merge"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <label className="text-sm text-text-secondary">Dictionary mode:</label>
            <Toggle
              checked={editValues.dictionaryMode === "override"}
              onCheckedChange={(checked) =>
                handleEditChange("dictionaryMode", checked ? "override" : "merge")
              }
              aria-label="Dictionary mode context only"
            />
            <span className="text-xs text-text-tertiary">
              {editValues.dictionaryMode === "override" ? "Context Only" : "Merge"}
            </span>
          </div>
        </div>
        {dictionaryOptions.length > 0 && (
          <FormField
            label="Dictionary Entries"
            help="Assign specific dictionary entries to this context."
            className="mt-3"
          >
            <MultiSelect
              selected={editValues.dictionaryEntryIds}
              onChange={(ids) => handleEditChange("dictionaryEntryIds", ids)}
              options={dictionaryOptions}
              placeholder="Select dictionary entries (optional)..."
              aria-label="Dictionary entries"
            />
          </FormField>
        )}
      </Card>
    );
  }

  if (isDeleting) {
    return (
      <Card className="p-3 border-error">
        <div className="flex items-center justify-between">
          <span className="text-text-secondary">Delete "{context.name}"?</span>
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
          <span className="font-medium text-text-primary shrink-0">{context.name}</span>
          <span className="text-text-secondary text-sm truncate">
            {context.matcher.appName}
            {context.matcher.titlePattern && (
              <span className="text-text-tertiary"> / {context.matcher.titlePattern}</span>
            )}
          </span>
          <div className="flex gap-2 shrink-0">
            {context.commandMode === "override" && (
              <span
                className="text-xs px-2 py-0.5 rounded bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300"
                title="Commands: Context Only mode"
              >
                Cmd
              </span>
            )}
            {context.dictionaryMode === "override" && (
              <span
                className="text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
                title="Dictionary: Context Only mode"
              >
                Dict
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
            onCheckedChange={(enabled) => handleToggleEnabled(context.id, enabled)}
            aria-label={`${context.enabled ? "Disable" : "Enable"} ${context.name}`}
          />
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleStartEdit(context)}
            aria-label={`Edit ${context.name}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => deletion.requestDelete(context.id)}
            aria-label={`Delete ${context.name}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Card>
  );
}
