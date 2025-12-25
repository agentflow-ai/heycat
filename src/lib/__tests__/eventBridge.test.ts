import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { setupEventBridge, eventNames } from "../eventBridge";
import { queryKeys } from "../queryKeys";
import type { AppState } from "../../stores/appStore";

// Store event handlers registered via listen()
type EventCallback = (event: { payload: unknown }) => void;
const eventHandlers: Map<string, EventCallback> = new Map();

// Mock unlisten functions
const mockUnlistenFns: Array<() => void> = [];
const createMockUnlisten = () => {
  const unlisten = vi.fn();
  mockUnlistenFns.push(unlisten);
  return unlisten;
};

// Mock the Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, callback: EventCallback) => {
    eventHandlers.set(eventName, callback);
    return Promise.resolve(createMockUnlisten());
  }),
}));

// Helper to emit a mock event
function emitMockEvent(eventName: string, payload: unknown = {}) {
  const handler = eventHandlers.get(eventName);
  if (handler) {
    handler({ payload });
  }
}

describe("eventBridge", () => {
  let queryClient: QueryClient;
  let mockStore: Pick<AppState, "setOverlayMode" | "transcriptionStarted" | "transcriptionCompleted" | "transcriptionError" | "wakeWordDetected" | "clearWakeWord">;

  beforeEach(() => {
    eventHandlers.clear();
    mockUnlistenFns.length = 0;
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });
    mockStore = {
      setOverlayMode: vi.fn() as AppState["setOverlayMode"],
      transcriptionStarted: vi.fn() as AppState["transcriptionStarted"],
      transcriptionCompleted: vi.fn() as AppState["transcriptionCompleted"],
      transcriptionError: vi.fn() as AppState["transcriptionError"],
      wakeWordDetected: vi.fn() as AppState["wakeWordDetected"],
      clearWakeWord: vi.fn() as AppState["clearWakeWord"],
    };
  });

  afterEach(() => {
    queryClient.clear();
  });

  // setupEventBridge tests removed per TESTING.md:
  // - Testing event listener registration/cleanup is testing framework internals
  // - User-visible behavior is tested in the event handler tests below

  describe("recording events trigger query invalidation", () => {
    it("recording_started invalidates getRecordingState query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.RECORDING_STARTED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    });

    it("recording_stopped invalidates getRecordingState query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.RECORDING_STOPPED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    });

    it("recording_error invalidates getRecordingState query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.RECORDING_ERROR);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    });

    it("recording_cancelled invalidates getRecordingState query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.RECORDING_CANCELLED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getRecordingState,
      });
    });
  });

  describe("listening events trigger query invalidation", () => {
    it("listening_started invalidates getListeningStatus query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.LISTENING_STARTED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    });

    it("listening_stopped invalidates getListeningStatus query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.LISTENING_STOPPED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    });

    it("listening_unavailable invalidates getListeningStatus query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.LISTENING_UNAVAILABLE);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.getListeningStatus,
      });
    });
  });

  describe("wake word events update store", () => {
    it("wake_word_detected updates store and auto-clears after 500ms", async () => {
      vi.useFakeTimers();
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.WAKE_WORD_DETECTED);

      expect(mockStore.wakeWordDetected).toHaveBeenCalled();
      expect(mockStore.clearWakeWord).not.toHaveBeenCalled();

      // Fast-forward 500ms
      vi.advanceTimersByTime(500);

      expect(mockStore.clearWakeWord).toHaveBeenCalled();

      vi.useRealTimers();
    });
  });

  describe("transcription events update store and query", () => {
    it("transcription_started updates store", async () => {
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.TRANSCRIPTION_STARTED);

      expect(mockStore.transcriptionStarted).toHaveBeenCalled();
    });

    it("transcription_completed updates store and invalidates listRecordings query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.TRANSCRIPTION_COMPLETED, {
        text: "Hello world",
        duration_ms: 1234,
      });

      expect(mockStore.transcriptionCompleted).toHaveBeenCalledWith(
        "Hello world",
        1234
      );
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.listRecordings,
      });
    });

    it("transcription_error updates store", async () => {
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.TRANSCRIPTION_ERROR, {
        error: "Model not loaded",
      });

      expect(mockStore.transcriptionError).toHaveBeenCalledWith(
        "Model not loaded"
      );
    });
  });

  describe("model events trigger query invalidation", () => {
    it("model_download_completed invalidates model status queries", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.MODEL_DOWNLOAD_COMPLETED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: ["tauri", "check_parakeet_model_status"],
      });
    });
  });

  describe("dictionary events trigger query invalidation", () => {
    it("dictionary_updated invalidates dictionary queries", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.DICTIONARY_UPDATED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.dictionary.all,
      });
    });
  });

  describe("hotkey events log warnings", () => {
    it("key_blocking_unavailable logs warning to console", async () => {
      const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.KEY_BLOCKING_UNAVAILABLE, {
        reason: "Accessibility permission denied",
        timestamp: "2025-01-01T12:00:00Z",
      });

      expect(warnSpy).toHaveBeenCalledWith(
        "[heycat] Key blocking unavailable:",
        "Accessibility permission denied",
        "- Escape key may propagate to other apps during recording cancel"
      );

      warnSpy.mockRestore();
    });
  });

  describe("UI state events update Zustand store", () => {
    it("overlay-mode event updates store with string payload", async () => {
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.OVERLAY_MODE, "recording");

      expect(mockStore.setOverlayMode).toHaveBeenCalledWith("recording");
    });

    it("overlay-mode event updates store with null payload", async () => {
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.OVERLAY_MODE, null);

      expect(mockStore.setOverlayMode).toHaveBeenCalledWith(null);
    });
  });
});
