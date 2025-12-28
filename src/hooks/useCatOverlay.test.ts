import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useCatOverlay } from "./useCatOverlay";
import { useAppStore } from "../stores/appStore";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();
const mockGetByLabel = vi.fn();
const mockPrimaryMonitor = vi.fn();

// Track callbacks for each event type
const eventCallbacks: Record<string, ((event: { payload: unknown }) => void)[]> = {};

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

// Mock useSettings hook
vi.mock("./useSettings", () => ({
  useSettings: () => ({
    settings: {
      audio: { selectedDevice: null },
      shortcuts: { distinguishLeftRight: false },
    },
    isLoading: false,
    updateAudioDevice: vi.fn(),
    updateDistinguishLeftRight: vi.fn(),
  }),
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

  it("initializes with isRecording: false and overlayMode: hidden", () => {
    const { result } = renderHook(() => useCatOverlay());

    expect(result.current.isRecording).toBe(false);
    expect(result.current.overlayMode).toBe("hidden");
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

  it("syncs overlay mode to Zustand store when recording", async () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(useAppStore.getState().overlayMode).toBe("recording");
    });
  });

  it("syncs overlay mode to Zustand store as null when hidden", async () => {
    mockUseRecording.mockReturnValue({
      isRecording: false,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    renderHook(() => useCatOverlay());

    await waitFor(() => {
      expect(useAppStore.getState().overlayMode).toBe(null);
    });
  });
});
