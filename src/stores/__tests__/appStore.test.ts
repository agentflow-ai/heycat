import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import {
  useAppStore,
  useOverlayMode,
  useSettingsCache,
  useIsSettingsLoaded,
} from "../appStore";
import type { AppSettings } from "../../hooks/useSettings";

const mockSettings: AppSettings = {
  listening: { enabled: true, autoStartOnLaunch: false },
  audio: { selectedDevice: "MacBook Pro Microphone" },
  shortcuts: { distinguishLeftRight: true },
};

describe("appStore", () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useAppStore.setState({
      overlayMode: null,
      settingsCache: null,
      isSettingsLoaded: false,
    });
  });

  describe("overlay mode", () => {
    it("setOverlayMode updates overlay state", () => {
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.setOverlayMode("recording");
      });

      expect(result.current.overlayMode).toBe("recording");
    });

    it("setOverlayMode(null) clears overlay state", () => {
      useAppStore.setState({ overlayMode: "commands" });
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.setOverlayMode(null);
      });

      expect(result.current.overlayMode).toBe(null);
    });
  });

  describe("settings cache", () => {
    it("setSettings caches settings and sets loaded flag", () => {
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.setSettings(mockSettings);
      });

      expect(result.current.settingsCache).toEqual(mockSettings);
      expect(result.current.isSettingsLoaded).toBe(true);
    });

    it("updateSetting updates a specific settings slice", () => {
      useAppStore.setState({
        settingsCache: mockSettings,
        isSettingsLoaded: true,
      });
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.updateSetting("audio", { selectedDevice: "External Mic" });
      });

      expect(result.current.settingsCache?.audio.selectedDevice).toBe(
        "External Mic"
      );
      // Other slices remain unchanged
      expect(result.current.settingsCache?.listening.enabled).toBe(true);
    });

    it("updateSetting does nothing when settingsCache is null", () => {
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.updateSetting("audio", { selectedDevice: "External Mic" });
      });

      expect(result.current.settingsCache).toBe(null);
    });
  });

  describe("selectors", () => {
    it("useOverlayMode returns only overlayMode slice", () => {
      useAppStore.setState({ overlayMode: "recording" });
      const { result } = renderHook(() => useOverlayMode());

      expect(result.current).toBe("recording");
    });

    it("useSettingsCache returns only settingsCache slice", () => {
      useAppStore.setState({ settingsCache: mockSettings, isSettingsLoaded: true });
      const { result } = renderHook(() => useSettingsCache());

      expect(result.current).toEqual(mockSettings);
    });

    it("useIsSettingsLoaded returns only isSettingsLoaded slice", () => {
      useAppStore.setState({ isSettingsLoaded: true });
      const { result } = renderHook(() => useIsSettingsLoaded());

      expect(result.current).toBe(true);
    });
  });
});
