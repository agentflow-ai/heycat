/**
 * Represents a dictionary entry for text expansion.
 * Matches the backend DictionaryEntry struct in src-tauri/src/dictionary/store.rs
 */
export interface DictionaryEntry {
  /** Unique identifier for the entry */
  id: string;
  /** Trigger word/phrase (e.g., "brb") */
  trigger: string;
  /** Expansion text (e.g., "be right back") */
  expansion: string;
  /** Optional suffix appended after expansion */
  suffix?: string;
  /** Whether to simulate enter keypress after expansion */
  autoEnter?: boolean;
}
