import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import {
  useAppStore,
  useOverlayMode,
  useSettingsCache,
  useIsSettingsLoaded,
  useTranscriptionState,
} from "../appStore";
import type { AppSettings } from "../../hooks/useSettings";

const mockSettings: AppSettings = {
  listening: { enabled: true, autoStartOnLaunch: false },
  audio: { selectedDevice: "MacBook Pro Microphone" },
  shortcuts: { distinguishLeftRight: true },
};

const initialTranscriptionState = {
  isTranscribing: false,
  transcribedText: null,
  error: null,
  durationMs: null,
};

describe("appStore", () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useAppStore.setState({
      overlayMode: null,
      settingsCache: null,
      isSettingsLoaded: false,
      transcription: initialTranscriptionState,
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

  // Selector tests removed per TESTING.md:
  // - Testing that selectors return correct slices is testing Zustand framework internals
  // - These are trivial one-liners that provide no value if they compile

  describe("transcription state", () => {
    it("transcriptionStarted sets isTranscribing and clears previous state", () => {
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.transcriptionStarted();
      });

      expect(result.current.transcription).toEqual({
        isTranscribing: true,
        transcribedText: null,
        error: null,
        durationMs: null,
      });
    });

    it("transcriptionCompleted sets result and clears isTranscribing", () => {
      useAppStore.setState({
        transcription: { ...initialTranscriptionState, isTranscribing: true },
      });
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.transcriptionCompleted("Hello world", 1234);
      });

      expect(result.current.transcription).toEqual({
        isTranscribing: false,
        transcribedText: "Hello world",
        error: null,
        durationMs: 1234,
      });
    });

    it("transcriptionError sets error and clears isTranscribing", () => {
      useAppStore.setState({
        transcription: { ...initialTranscriptionState, isTranscribing: true },
      });
      const { result } = renderHook(() => useAppStore());

      act(() => {
        result.current.transcriptionError("Model not loaded");
      });

      expect(result.current.transcription.isTranscribing).toBe(false);
      expect(result.current.transcription.error).toBe("Model not loaded");
    });
  });
});
