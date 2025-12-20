import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useCatOverlay } from "./useCatOverlay";
import { useAppStore } from "../stores/appStore";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();
const mockGetByLabel = vi.fn();
const mockPrimaryMonitor = vi.fn();
const mockInvoke = vi.fn();

// Track callbacks for each event type
const eventCallbacks: Record<string, ((event: { payload: unknown }) => void)[]> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => mockInvoke(cmd),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (eventName: string, callback: (event: { payload: unknown }) => void) => {
    mockListen(eventName, callback);
    if (!eventCallbacks[eventName]) {
      eventCallbacks[eventName] = [];
    }
    eventCallbacks[eventName].push(callback);
    return Promise.resolve(mockUnlisten);
  },
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: class MockWebviewWindow {
    static getByLabel = (label: string) => mockGetByLabel(label);
    constructor() {
      // Mock constructor
    }
    once() {
      // Mock once
    }
  },
}));

vi.mock("@tauri-apps/api/window", () => ({
  primaryMonitor: () => mockPrimaryMonitor(),
  LogicalPosition: class MockLogicalPosition {
    constructor(
      public x: number,
      public y: number
    ) {}
  },
}));

// Mock useRecording hook
const mockUseRecording = vi.fn();
vi.mock("./useRecording", () => ({
  useRecording: () => mockUseRecording(),
}));

describe("useCatOverlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset Zustand store to initial state
    useAppStore.setState({
      overlayMode: null,
      settingsCache: null,
      isSettingsLoaded: false,
    });
    // Clear event callbacks
    Object.keys(eventCallbacks).forEach((key) => {
      delete eventCallbacks[key];
    });
    // Default mocks
    mockListen.mockResolvedValue(mockUnlisten);
    mockGetByLabel.mockResolvedValue(null);
    mockPrimaryMonitor.mockResolvedValue({
      position: { x: 0, y: 0 },
      size: { width: 1920, height: 1080 },
      scaleFactor: 1,
    });
    // Default: listening is disabled
    mockInvoke.mockResolvedValue({
      enabled: false,
      active: false,
      micAvailable: true,
    });
    mockUseRecording.mockReturnValue({
      isRecording: false,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with isListening: false and overlayMode: hidden", () => {
    const { result } = renderHook(() => useCatOverlay());

    expect(result.current.isListening).toBe(false);
    expect(result.current.isRecording).toBe(false);
    expect(result.current.overlayMode).toBe("hidden");
    expect(result.current.isMicUnavailable).toBe(false);
  });

  it("fetches initial listening state from backend on mount", async () => {
    // Backend reports listening is already enabled
    mockInvoke.mockResolvedValue({
      enabled: true,
      active: true,
      micAvailable: true,
    });

    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_listening_status");
    });

    await waitFor(() => {
      expect(result.current.isListening).toBe(true);
      expect(result.current.overlayMode).toBe("listening");
    });
  });

  it("sets isMicUnavailable from initial backend state", async () => {
    mockInvoke.mockResolvedValue({
      enabled: false,
      active: false,
      micAvailable: false,
    });

    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(result.current.isMicUnavailable).toBe(true);
    });
  });

  it("sets up listening event listeners on mount", async () => {
    renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "listening_started",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "listening_stopped",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "wake_word_detected",
        expect.any(Function)
      );
      expect(mockListen).toHaveBeenCalledWith(
        "listening_unavailable",
        expect.any(Function)
      );
    });
  });

  it("updates isListening to true on listening_started event", async () => {
    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_started"]).toBeDefined();
    });

    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:00:00Z" },
      });
    });

    expect(result.current.isListening).toBe(true);
    expect(result.current.overlayMode).toBe("listening");
  });

  it("updates isListening to false on listening_stopped event", async () => {
    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_started"]).toBeDefined();
      expect(eventCallbacks["listening_stopped"]).toBeDefined();
    });

    // First start listening
    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:00:00Z" },
      });
    });
    expect(result.current.isListening).toBe(true);

    // Then stop listening
    act(() => {
      eventCallbacks["listening_stopped"][0]({
        payload: { timestamp: "2025-01-01T12:01:00Z" },
      });
    });
    expect(result.current.isListening).toBe(false);
    expect(result.current.overlayMode).toBe("hidden");
  });

  it("sets isMicUnavailable on listening_unavailable event", async () => {
    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_unavailable"]).toBeDefined();
    });

    act(() => {
      eventCallbacks["listening_unavailable"][0]({
        payload: { reason: "Microphone disconnected", timestamp: "2025-01-01T12:00:00Z" },
      });
    });

    expect(result.current.isMicUnavailable).toBe(true);
  });

  it("clears isMicUnavailable when listening starts again", async () => {
    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_unavailable"]).toBeDefined();
      expect(eventCallbacks["listening_started"]).toBeDefined();
    });

    // First set mic unavailable
    act(() => {
      eventCallbacks["listening_unavailable"][0]({
        payload: { reason: "Microphone disconnected", timestamp: "2025-01-01T12:00:00Z" },
      });
    });
    expect(result.current.isMicUnavailable).toBe(true);

    // Then start listening - mic becomes available again
    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:01:00Z" },
      });
    });
    expect(result.current.isMicUnavailable).toBe(false);
  });

  it("overlayMode is 'recording' when isRecording is true", async () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    const { result } = renderHook(() => useCatOverlay());

    expect(result.current.overlayMode).toBe("recording");
  });

  it("overlayMode is 'recording' even when isListening is also true (recording takes precedence)", async () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_started"]).toBeDefined();
    });

    // Set listening to true
    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:00:00Z" },
      });
    });

    // Recording should take precedence
    expect(result.current.isListening).toBe(true);
    expect(result.current.isRecording).toBe(true);
    expect(result.current.overlayMode).toBe("recording");
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      // 4 listening events
      expect(mockListen).toHaveBeenCalledTimes(4);
    });

    unmount();

    // Each listener's unlisten function should be called
    expect(mockUnlisten).toHaveBeenCalledTimes(4);
  });

  it("wake_word_detected event is received but doesn't change state directly", async () => {
    const { result } = renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["wake_word_detected"]).toBeDefined();
    });

    const initialState = { ...result.current };

    act(() => {
      eventCallbacks["wake_word_detected"][0]({
        payload: {
          confidence: 0.95,
          transcription: "hey cat",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });

    // State should be unchanged - recording_started will follow
    expect(result.current.isListening).toBe(initialState.isListening);
  });

  it("syncs overlay mode to Zustand store when listening starts", async () => {
    renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_started"]).toBeDefined();
    });

    // Initially store should have null overlayMode
    expect(useAppStore.getState().overlayMode).toBe(null);

    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:00:00Z" },
      });
    });

    // Store should be updated to "listening"
    expect(useAppStore.getState().overlayMode).toBe("listening");
  });

  it("syncs overlay mode to Zustand store as null when hidden", async () => {
    renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(eventCallbacks["listening_started"]).toBeDefined();
      expect(eventCallbacks["listening_stopped"]).toBeDefined();
    });

    // Start listening
    act(() => {
      eventCallbacks["listening_started"][0]({
        payload: { timestamp: "2025-01-01T12:00:00Z" },
      });
    });
    expect(useAppStore.getState().overlayMode).toBe("listening");

    // Stop listening - should go back to null
    act(() => {
      eventCallbacks["listening_stopped"][0]({
        payload: { timestamp: "2025-01-01T12:01:00Z" },
      });
    });
    expect(useAppStore.getState().overlayMode).toBe(null);
  });
});
