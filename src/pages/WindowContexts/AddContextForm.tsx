import { useState, useMemo, useCallback } from "react";
import { Plus } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, Combobox, MultiSelect, Toggle } from "../../components/ui";
import type { ComboboxOption } from "../../components/ui";
import type { OverrideMode } from "../../types/windowContext";
import { validateRegexPattern } from "../../lib/validation";
import { useWindowContextsContext } from "./WindowContextsContext";

/**
 * Form for adding new window contexts.
 */
export function AddContextForm() {
  const { handleAddContext, runningApps, dictionaryEntries, dictionaryOptions } = useWindowContextsContext();

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

  const handleAppSelect = useCallback((option: ComboboxOption) => {
    setAppName(option.value);
    setBundleId(option.description);
    setAppNameError(null);
  }, []);

  const handlePatternValidation = (pattern: string): boolean => {
    const error = validateRegexPattern(pattern);
    setPatternError(error);
    return error === null;
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

    if (!handlePatternValidation(titlePattern)) {
      return;
    }

    setIsSubmitting(true);
    try {
      await handleAddContext({
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
                placeholder="Select or type app name..."
                aria-label="App name"
              />
            </FormField>
            <FormField label="Title Pattern" error={patternError ?? undefined} className="flex-1 min-w-[150px]">
              <Input
                type="text"
                placeholder="e.g., #general.*"
                value={titlePattern}
                onChange={(e) => {
                  setTitlePattern(e.target.value);
                  handlePatternValidation(e.target.value);
                }}
                aria-label="Window title pattern (regex)"
              />
            </FormField>
            <div className="pt-6">
              <Button type="submit" disabled={isSubmitting}>
                <Plus className="h-4 w-4" />
                Add
              </Button>
            </div>
          </div>
          <div className="mt-3 flex gap-4 items-center">
            <div className="flex items-center gap-2">
              <label className="text-sm text-text-secondary">Command mode:</label>
              <Toggle
                checked={commandMode === "override"}
                onCheckedChange={(checked) => setCommandMode(checked ? "override" : "merge")}
                aria-label="Command mode context only"
              />
              <span className="text-xs text-text-tertiary">
                {commandMode === "override" ? "Context Only" : "Merge"}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <label className="text-sm text-text-secondary">Dictionary mode:</label>
              <Toggle
                checked={dictionaryMode === "override"}
                onCheckedChange={(checked) => setDictionaryMode(checked ? "override" : "merge")}
                aria-label="Dictionary mode context only"
              />
              <span className="text-xs text-text-tertiary">
                {dictionaryMode === "override" ? "Context Only" : "Merge"}
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
                selected={selectedDictionaryIds}
                onChange={setSelectedDictionaryIds}
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
