import { useMemo } from "react";
import { Plus } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, Combobox, MultiSelect, Toggle } from "../../components/ui";
import type { ComboboxOption } from "../../components/ui";
import { useWindowContextForm } from "../../hooks/useWindowContextForm";
import { useWindowContextsContext } from "./WindowContextsContext";

/**
 * Form for adding new window contexts.
 * Uses useWindowContextForm for state and validation.
 */
export function AddContextForm() {
  const { handleAddContext, runningApps, dictionaryEntries, dictionaryOptions } = useWindowContextsContext();

  const form = useWindowContextForm({
    onSubmit: async (values) => {
      await handleAddContext({
        name: values.name.trim(),
        appName: values.appName.trim(),
        bundleId: values.bundleId,
        titlePattern: values.titlePattern.trim() || undefined,
        commandMode: values.commandMode,
        dictionaryMode: values.dictionaryMode,
        dictionaryEntryIds: values.dictionaryEntryIds,
        priority: values.priority,
        enabled: values.enabled,
      });
      form.reset();
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

  const handleAppSelect = (option: ComboboxOption) => {
    form.handleAppSelect(option.value, option.description);
  };

  return (
    <Card>
      <CardContent className="p-4">
        <form onSubmit={form.handleSubmit}>
          <div className="flex gap-3 items-start flex-wrap">
            <FormField label="Name" error={form.nameError ?? undefined} className="flex-1 min-w-[150px]">
              <Input
                type="text"
                placeholder="e.g., Slack General"
                value={form.values.name}
                onChange={(e) => form.setValue("name", e.target.value)}
                aria-label="Context name"
              />
            </FormField>
            <FormField label="App Name" error={form.appNameError ?? undefined} className="flex-1 min-w-[150px]">
              <Combobox
                value={form.values.appName}
                onChange={(value) => form.setValue("appName", value)}
                onSelect={handleAppSelect}
                options={appOptions}
                placeholder="Select or type app name..."
                aria-label="App name"
              />
            </FormField>
            <FormField label="Title Pattern" error={form.patternError ?? undefined} className="flex-1 min-w-[150px]">
              <Input
                type="text"
                placeholder="e.g., #general.*"
                value={form.values.titlePattern}
                onChange={(e) => form.setValue("titlePattern", e.target.value)}
                aria-label="Window title pattern (regex)"
              />
            </FormField>
            <div className="pt-6">
              <Button type="submit" disabled={form.isSubmitting}>
                <Plus className="h-4 w-4" />
                Add
              </Button>
            </div>
          </div>
          <div className="mt-3 flex gap-4 items-center">
            <div className="flex items-center gap-2">
              <label className="text-sm text-text-secondary">Command mode:</label>
              <Toggle
                checked={form.values.commandMode === "replace"}
                onCheckedChange={(checked) => form.setValue("commandMode", checked ? "replace" : "merge")}
                aria-label="Command mode context only"
              />
              <span className="text-xs text-text-tertiary">
                {form.values.commandMode === "replace" ? "Context Only" : "Merge"}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <label className="text-sm text-text-secondary">Dictionary mode:</label>
              <Toggle
                checked={form.values.dictionaryMode === "replace"}
                onCheckedChange={(checked) => form.setValue("dictionaryMode", checked ? "replace" : "merge")}
                aria-label="Dictionary mode context only"
              />
              <span className="text-xs text-text-tertiary">
                {form.values.dictionaryMode === "replace" ? "Context Only" : "Merge"}
              </span>
            </div>
          </div>
          {dictionaryEntries.length > 0 && (
            <FormField
              label="Dictionary Entries"
              help="Assign specific dictionary entries to this context."
              className="mt-3"
            >
              <MultiSelect
                selected={form.values.dictionaryEntryIds}
                onChange={(ids) => form.setValue("dictionaryEntryIds", ids)}
                options={dictionaryOptions}
                placeholder="Select dictionary entries (optional)..."
                aria-label="Dictionary entries"
              />
            </FormField>
          )}
        </form>
      </CardContent>
    </Card>
  );
}
