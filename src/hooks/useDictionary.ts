import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation } from "@tanstack/react-query";
import { queryKeys } from "../lib/queryKeys";
import type { DictionaryEntry } from "../types/dictionary";

/**
 * Hook for dictionary CRUD operations.
 *
 * Uses Tanstack Query for data fetching and mutations, following the existing
 * patterns in the codebase. Provides optimistic updates and error handling.
 *
 * Note: Mutations do NOT invalidate queries in onSuccess. The Event Bridge
 * handles cache invalidation when the backend emits dictionary_updated events.
 */
export function useDictionary() {
  const entries = useQuery({
    queryKey: queryKeys.dictionary.list(),
    queryFn: () => invoke<DictionaryEntry[]>("list_dictionary_entries"),
  });

  const addEntry = useMutation({
    mutationFn: (data: {
      trigger: string;
      expansion: string;
      suffix?: string;
      autoEnter?: boolean;
      disableSuffix?: boolean;
      completeMatchOnly?: boolean;
    }) =>
      invoke<DictionaryEntry>("add_dictionary_entry", {
        trigger: data.trigger,
        expansion: data.expansion,
        suffix: data.suffix,
        autoEnter: data.autoEnter,
        disableSuffix: data.disableSuffix,
        completeMatchOnly: data.completeMatchOnly,
      }),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  const updateEntry = useMutation({
    mutationFn: (data: {
      id: string;
      trigger: string;
      expansion: string;
      suffix?: string;
      autoEnter?: boolean;
      disableSuffix?: boolean;
      completeMatchOnly?: boolean;
    }) =>
      invoke<void>("update_dictionary_entry", {
        id: data.id,
        trigger: data.trigger,
        expansion: data.expansion,
        suffix: data.suffix,
        autoEnter: data.autoEnter,
        disableSuffix: data.disableSuffix,
        completeMatchOnly: data.completeMatchOnly,
      }),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  const deleteEntry = useMutation({
    mutationFn: (data: { id: string }) =>
      invoke<void>("delete_dictionary_entry", data),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  return { entries, addEntry, updateEntry, deleteEntry };
}
