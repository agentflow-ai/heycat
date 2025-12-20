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
  let mockStore: Pick<AppState, "setOverlayMode">;

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
    };
  });

  afterEach(() => {
    queryClient.clear();
  });

  describe("setupEventBridge", () => {
    it("returns a cleanup function", async () => {
      const cleanup = await setupEventBridge(queryClient, mockStore);
      expect(typeof cleanup).toBe("function");
    });

    it("registers all expected event listeners", async () => {
      await setupEventBridge(queryClient, mockStore);

      expect(eventHandlers.has(eventNames.RECORDING_STARTED)).toBe(true);
      expect(eventHandlers.has(eventNames.RECORDING_STOPPED)).toBe(true);
      expect(eventHandlers.has(eventNames.RECORDING_ERROR)).toBe(true);
      expect(eventHandlers.has(eventNames.TRANSCRIPTION_COMPLETED)).toBe(true);
      expect(eventHandlers.has(eventNames.LISTENING_STARTED)).toBe(true);
      expect(eventHandlers.has(eventNames.LISTENING_STOPPED)).toBe(true);
      expect(eventHandlers.has(eventNames.MODEL_DOWNLOAD_COMPLETED)).toBe(true);
      expect(eventHandlers.has(eventNames.OVERLAY_MODE)).toBe(true);
    });

    it("cleanup function unsubscribes all listeners", async () => {
      const cleanup = await setupEventBridge(queryClient, mockStore);

      // Should have registered 8 listeners
      expect(mockUnlistenFns.length).toBe(8);

      // Call cleanup
      cleanup();

      // All unlisten functions should have been called
      mockUnlistenFns.forEach((unlisten) => {
        expect(unlisten).toHaveBeenCalledTimes(1);
      });
    });
  });

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
  });

  describe("transcription events trigger query invalidation", () => {
    it("transcription_completed invalidates listRecordings query", async () => {
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
      await setupEventBridge(queryClient, mockStore);

      emitMockEvent(eventNames.TRANSCRIPTION_COMPLETED);

      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: queryKeys.tauri.listRecordings,
      });
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
