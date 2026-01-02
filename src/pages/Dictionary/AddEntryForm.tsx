import { useState } from "react";
import { Plus, Settings } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, MultiSelect } from "../../components/ui";
import type { MultiSelectOption } from "../../components/ui";
import { validateSuffix } from "../../lib/validation";
import { EntrySettings } from "./EntrySettings";
import { useDictionaryContext } from "./DictionaryContext";

/**
 * Form for adding new dictionary entries.
 * Uses DictionaryContext for submission and validation.
 */
export function AddEntryForm() {
  const { handleAddEntry, existingTriggers, contextOptions } = useDictionaryContext();

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

  const handleSuffixValidation = (value: string): boolean => {
    const error = validateSuffix(value);
    if (error) {
      setSuffixError(error);
      return false;
    }
    setSuffixError(null);
    return true;
  };

  const handleSuffixChange = (value: string) => {
    setSuffix(value);
    handleSuffixValidation(value);
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
    if (!handleSuffixValidation(suffix)) {
      return;
    }

    setIsSubmitting(true);
    try {
      await handleAddEntry(
        trigger.trim(),
        expansion.trim(),
        selectedContextIds,
        disableSuffix ? undefined : suffix.trim() || undefined,
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
            <FormField label="Trigger" error={triggerError ?? undefined} className="flex-1">
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
              <EntrySettings
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
