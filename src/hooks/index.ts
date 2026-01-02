/**
 * Hooks barrel export.
 *
 * This module exports all React hooks for easy importing.
 */

// Generic/reusable hooks
export { useFormState } from "./useFormState";
export type {
  UseFormStateOptions,
  UseFormStateReturn,
} from "./useFormState";

export { useEditableItem } from "./useEditableItem";
export type {
  UseEditableItemOptions,
  UseEditableItemReturn,
} from "./useEditableItem";

export { useDeleteConfirmation } from "./useDeleteConfirmation";
export type {
  UseDeleteConfirmationOptions,
  UseDeleteConfirmationReturn,
} from "./useDeleteConfirmation";

export { useSearch } from "./useSearch";
export type {
  UseSearchOptions,
  UseSearchReturn,
} from "./useSearch";

// Feature-specific hooks
export { useDictionaryForm } from "./useDictionaryForm";
export type {
  UseDictionaryFormOptions,
  UseDictionaryFormReturn,
  DictionaryFormValues,
} from "./useDictionaryForm";

export { useWindowContextForm } from "./useWindowContextForm";
export type {
  UseWindowContextFormOptions,
  UseWindowContextFormReturn,
  WindowContextFormValues,
} from "./useWindowContextForm";

export { useRecordingsFilter } from "./useRecordingsFilter";
export type {
  UseRecordingsFilterOptions,
  UseRecordingsFilterReturn,
  FilterOption,
  SortOption,
} from "./useRecordingsFilter";

export { useShortcutRecorder } from "./useShortcutRecorder";
export type { UseShortcutRecorderReturn } from "./useShortcutRecorder";

// Existing domain hooks
export { useActiveWindow } from "./useActiveWindow";
export { useAppStatus } from "./useAppStatus";
export { useAudioDevices } from "./useAudioDevices";
export { useAudioLevelMonitor } from "./useAudioLevelMonitor";
export { useAudioPlayback } from "./useAudioPlayback";
export { useCatOverlay } from "./useCatOverlay";
export { useDictionary } from "./useDictionary";
export { useDisambiguation } from "./useDisambiguation";
export { useMultiModelStatus } from "./useMultiModelStatus";
export { useRecording } from "./useRecording";
export { useSettings } from "./useSettings";
export { useTranscription } from "./useTranscription";
export { useWindowContext } from "./useWindowContext";
