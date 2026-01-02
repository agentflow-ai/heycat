import { Input, Toggle } from "../../components/ui";

export interface EntrySettingsProps {
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

/**
 * Settings panel for dictionary entry options.
 * Used in both add and edit forms.
 */
export function EntrySettings({
  suffix,
  autoEnter,
  disableSuffix,
  completeMatchOnly,
  onSuffixChange,
  onAutoEnterChange,
  onDisableSuffixChange,
  onCompleteMatchOnlyChange,
  suffixError,
}: EntrySettingsProps) {
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
            {suffixError && <span className="text-error text-xs">{suffixError}</span>}
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
