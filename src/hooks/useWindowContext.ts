import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation } from "@tanstack/react-query";
import { queryKeys } from "../lib/queryKeys";
import type { WindowContext, RunningApplication } from "../types/windowContext";

/**
 * Hook for window context CRUD operations.
 *
 * Uses Tanstack Query for data fetching and mutations, following the existing
 * patterns in the codebase. Provides optimistic updates and error handling.
 *
 * Note: Mutations do NOT invalidate queries in onSuccess. The Event Bridge
 * handles cache invalidation when the backend emits window_contexts_updated events.
 */
export function useWindowContext() {
  const contexts = useQuery({
    queryKey: queryKeys.windowContext.list(),
    queryFn: () => invoke<WindowContext[]>("list_window_contexts"),
  });

  const addContext = useMutation({
    mutationFn: (data: {
      name: string;
      appName: string;
      titlePattern?: string;
      bundleId?: string;
      commandMode?: string;
      dictionaryMode?: string;
      commandIds?: string[];
      dictionaryEntryIds?: string[];
      enabled?: boolean;
      priority?: number;
    }) =>
      invoke<WindowContext>("add_window_context", {
        name: data.name,
        appName: data.appName,
        titlePattern: data.titlePattern,
        bundleId: data.bundleId,
        commandMode: data.commandMode,
        dictionaryMode: data.dictionaryMode,
        commandIds: data.commandIds,
        dictionaryEntryIds: data.dictionaryEntryIds,
        enabled: data.enabled,
        priority: data.priority,
      }),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  const updateContext = useMutation({
    mutationFn: (data: {
      id: string;
      name: string;
      appName: string;
      titlePattern?: string;
      bundleId?: string;
      commandMode?: string;
      dictionaryMode?: string;
      commandIds?: string[];
      dictionaryEntryIds?: string[];
      enabled?: boolean;
      priority?: number;
    }) =>
      invoke<void>("update_window_context", {
        id: data.id,
        name: data.name,
        appName: data.appName,
        titlePattern: data.titlePattern,
        bundleId: data.bundleId,
        commandMode: data.commandMode,
        dictionaryMode: data.dictionaryMode,
        commandIds: data.commandIds,
        dictionaryEntryIds: data.dictionaryEntryIds,
        enabled: data.enabled,
        priority: data.priority,
      }),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  const deleteContext = useMutation({
    mutationFn: (data: { id: string }) =>
      invoke<void>("delete_window_context", data),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  return { contexts, addContext, updateContext, deleteContext };
}

/**
 * Hook for fetching running applications.
 *
 * Returns a list of user-visible applications currently running on macOS.
 * Background helpers, agents, and daemons are filtered out by the backend.
 */
export function useRunningApplications() {
  return useQuery({
    queryKey: queryKeys.windowContext.runningApps(),
    queryFn: () => invoke<RunningApplication[]>("list_running_applications"),
    // Refetch on focus since running apps can change frequently
    refetchOnWindowFocus: true,
    // Stale after 30 seconds
    staleTime: 30_000,
  });
}
