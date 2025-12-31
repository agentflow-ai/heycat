import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSettings, initializeSettings, DEFAULT_SETTINGS } from "./useSettings";
import { useAppStore } from "../stores/appStore";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  },
}));

// Mock Tauri store plugin
vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

describe("useSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStore.get.mockResolvedValue(undefined);
    // Reset Zustand store to initial state
    useAppStore.setState({
      settingsCache: null,
      isSettingsLoaded: false,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("initializeSettings", () => {
    it("loads settings from Tauri Store into Zustand", async () => {
      mockStore.get.mockImplementation((key: string) => {
        if (key === "audio.selectedDevice") return Promise.resolve("USB Microphone");
        if (key === "shortcuts.distinguishLeftRight") return Promise.resolve(true);
        if (key === "hotkey.recordingMode") return Promise.resolve("push-to-talk");
        return Promise.resolve(undefined);
      });

      await initializeSettings();

      // Verify Zustand store was updated
      const state = useAppStore.getState();
      expect(state.isSettingsLoaded).toBe(true);
      expect(state.settingsCache).toEqual({
        audio: { selectedDevice: "USB Microphone" },
        shortcuts: { distinguishLeftRight: true, recordingMode: "push-to-talk" },
      });
    });

    it("uses defaults when Tauri Store has no values", async () => {
      mockStore.get.mockResolvedValue(undefined);

      await initializeSettings();

      const state = useAppStore.getState();
      expect(state.isSettingsLoaded).toBe(true);
      expect(state.settingsCache).toEqual(DEFAULT_SETTINGS);
    });
  });

  describe("useSettings hook", () => {
    it("returns settings from Zustand when loaded", async () => {
      // Pre-populate Zustand with settings
      useAppStore.setState({
        settingsCache: {
          audio: { selectedDevice: "My Mic" },
          shortcuts: { distinguishLeftRight: false, recordingMode: "toggle" },
        },
        isSettingsLoaded: true,
      });

      const { result } = renderHook(() => useSettings());

      expect(result.current.isLoading).toBe(false);
      expect(result.current.settings.audio.selectedDevice).toBe("My Mic");
    });

    it("returns defaults when settings not yet loaded", () => {
      const { result } = renderHook(() => useSettings());

      expect(result.current.isLoading).toBe(true);
      expect(result.current.settings).toEqual(DEFAULT_SETTINGS);
    });

    it("updates audio device in both stores", async () => {
      useAppStore.setState({
        settingsCache: {
          audio: { selectedDevice: null },
          shortcuts: { distinguishLeftRight: false, recordingMode: "toggle" },
        },
        isSettingsLoaded: true,
      });

      const { result } = renderHook(() => useSettings());

      await act(async () => {
        await result.current.updateAudioDevice("USB Microphone");
      });

      expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", "USB Microphone");
      expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");
    });

    it("clears audio device selection", async () => {
      useAppStore.setState({
        settingsCache: {
          audio: { selectedDevice: "USB Microphone" },
          shortcuts: { distinguishLeftRight: false, recordingMode: "toggle" },
        },
        isSettingsLoaded: true,
      });

      const { result } = renderHook(() => useSettings());

      await act(async () => {
        await result.current.updateAudioDevice(null);
      });

      expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", null);
      expect(result.current.settings.audio.selectedDevice).toBeNull();
    });
  });
});
