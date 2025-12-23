/**
 * Central event bridge that routes Tauri backend events to appropriate state managers.
 *
 * Event types and their destinations:
 * - Server state events → Tanstack Query invalidation (triggers refetch)
 * - UI state events → Zustand store updates (direct state mutation)
 *
 * This is the integration layer between backend-initiated events and frontend state.
 * All event subscriptions are set up once on app mount and cleaned up on unmount.
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { QueryClient } from "@tanstack/react-query";
import { queryKeys } from "./queryKeys";
import type { AppState } from "../stores/appStore";

/**
 * Event names emitted by the Rust backend.
 * These must match the constants in src-tauri/src/events.rs
 */
export const eventNames = {
  // Recording events
  RECORDING_STARTED: "recording_started",
  RECORDING_STOPPED: "recording_stopped",
  RECORDING_CANCELLED: "recording_cancelled",
  RECORDING_ERROR: "recording_error",

  // Listening events
  LISTENING_STARTED: "listening_started",
  LISTENING_STOPPED: "listening_stopped",
  LISTENING_UNAVAILABLE: "listening_unavailable",
  WAKE_WORD_DETECTED: "wake_word_detected",

  // Transcription events
  TRANSCRIPTION_STARTED: "transcription_started",
  TRANSCRIPTION_COMPLETED: "transcription_completed",
  TRANSCRIPTION_ERROR: "transcription_error",

  // Model events
  MODEL_DOWNLOAD_COMPLETED: "model_download_completed",

  // Dictionary events
  DICTIONARY_UPDATED: "dictionary_updated",

  // Hotkey events
  KEY_BLOCKING_UNAVAILABLE: "key_blocking_unavailable",

  // UI state events
  OVERLAY_MODE: "overlay-mode",
} as const;

/**
 * Payload type for overlay-mode event.
 * The mode can be null to indicate no overlay should be shown.
 */
export type OverlayModePayload = string | null;

/** Payload for transcription_completed event */
export interface TranscriptionCompletedPayload {
  text: string;
  duration_ms: number;
}

/** Payload for transcription_error event */
export interface TranscriptionErrorPayload {
  error: string;
}

/** Payload for key_blocking_unavailable event */
export interface KeyBlockingUnavailablePayload {
  reason: string;
  timestamp: string;
}

/**
 * Sets up the central event bridge that routes Tauri events to state managers.
 *
 * @param queryClient - Tanstack Query client for cache invalidation
 * @param store - Zustand store state for UI updates
 * @returns Cleanup function that unsubscribes all event listeners
 *
 * @example
 * ```typescript
 * const cleanup = await setupEventBridge(queryClient, useAppStore.getState());
 * // On unmount:
 * cleanup();
 * ```
 */
export async function setupEventBridge(
  queryClient: QueryClient,
  store: Pick<AppState, "setOverlayMode" | "transcriptionStarted" | "transcriptionCompleted" | "transcriptionError" | "wakeWordDetected" | "clearWakeWord">
): Promise<() => void> {
  const unlistenFns: UnlistenFn[] = [];

  // ============================================================
  // Server state events → Query invalidation
  // ============================================================

  // Recording state events - invalidate recording state query
  unlistenFns.push(
    await listen(eventNames.RECORDING_STARTED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.RECORDING_STOPPED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.RECORDING_ERROR, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.RECORDING_CANCELLED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    })
  );

  // Note: TRANSCRIPTION_COMPLETED is handled below with store update AND query invalidation

  // Listening state events - invalidate listening status query
  unlistenFns.push(
    await listen(eventNames.LISTENING_STARTED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    })
  );

  unlistenFns.push(
    await listen(eventNames.LISTENING_STOPPED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    })
  );

  // Listening unavailable - invalidate listening status query (mic not available)
  unlistenFns.push(
    await listen(eventNames.LISTENING_UNAVAILABLE, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    })
  );

  // Model events - invalidate all model status queries
  // Using partial match since model status queries have a type parameter
  unlistenFns.push(
    await listen(eventNames.MODEL_DOWNLOAD_COMPLETED, () => {
      queryClient.invalidateQueries({
        queryKey: ["tauri", "check_parakeet_model_status"],
      });
    })
  );

  // Dictionary events - invalidate dictionary list query
  unlistenFns.push(
    await listen(eventNames.DICTIONARY_UPDATED, () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.dictionary.all,
      });
    })
  );

  // Hotkey events - log warnings for edge cases
  unlistenFns.push(
    await listen<KeyBlockingUnavailablePayload>(eventNames.KEY_BLOCKING_UNAVAILABLE, (event) => {
      console.warn(
        "[heycat] Key blocking unavailable:",
        event.payload.reason,
        "- Escape key may propagate to other apps during recording cancel"
      );
    })
  );

  // ============================================================
  // UI state events → Zustand updates
  // ============================================================

  // Overlay mode changes - update Zustand store directly
  unlistenFns.push(
    await listen<OverlayModePayload>(eventNames.OVERLAY_MODE, (event) => {
      store.setOverlayMode(event.payload);
    })
  );

  // Transcription events - update Zustand store directly
  unlistenFns.push(
    await listen(eventNames.TRANSCRIPTION_STARTED, () => {
      store.transcriptionStarted();
    })
  );

  unlistenFns.push(
    await listen<TranscriptionCompletedPayload>(eventNames.TRANSCRIPTION_COMPLETED, (event) => {
      store.transcriptionCompleted(event.payload.text, event.payload.duration_ms);
      // Also invalidate recordings list since transcription produces a recording
      queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.listRecordings,
      });
    })
  );

  unlistenFns.push(
    await listen<TranscriptionErrorPayload>(eventNames.TRANSCRIPTION_ERROR, (event) => {
      store.transcriptionError(event.payload.error);
    })
  );

  // Wake word detected - transient UI state with auto-clear after 500ms
  unlistenFns.push(
    await listen(eventNames.WAKE_WORD_DETECTED, () => {
      store.wakeWordDetected();
      // Auto-clear after 500ms for transient visual feedback
      setTimeout(() => {
        store.clearWakeWord();
      }, 500);
    })
  );

  // Return cleanup function that unsubscribes all listeners
  return () => {
    unlistenFns.forEach((unlisten) => unlisten());
  };
}
