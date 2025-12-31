import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../appStore";

describe("appStore", () => {
  beforeEach(() => {
    // Reset the store to initial state before each test
    useAppStore.setState({
      overlayMode: null,
      settingsCache: null,
      isSettingsLoaded: false,
      transcription: {
        isTranscribing: false,
        transcribedText: null,
        error: null,
        durationMs: null,
      },
    });
  });

  describe("setOverlayMode", () => {
    it("sets overlay mode to recording", () => {
      useAppStore.getState().setOverlayMode("recording");
      expect(useAppStore.getState().overlayMode).toBe("recording");
    });

    it("sets overlay mode to commands", () => {
      useAppStore.getState().setOverlayMode("commands");
      expect(useAppStore.getState().overlayMode).toBe("commands");
    });

    it("clears overlay mode when set to null", () => {
      useAppStore.getState().setOverlayMode("recording");
      expect(useAppStore.getState().overlayMode).toBe("recording");

      useAppStore.getState().setOverlayMode(null);
      expect(useAppStore.getState().overlayMode).toBeNull();
    });

    it("transitions between overlay modes", () => {
      useAppStore.getState().setOverlayMode("recording");
      expect(useAppStore.getState().overlayMode).toBe("recording");

      useAppStore.getState().setOverlayMode("commands");
      expect(useAppStore.getState().overlayMode).toBe("commands");
    });
  });

  describe("transcription state", () => {
    it("transcriptionStarted sets isTranscribing and clears previous state", () => {
      // Set up previous state
      useAppStore.setState({
        transcription: {
          isTranscribing: false,
          transcribedText: "Previous text",
          error: "Previous error",
          durationMs: 500,
        },
      });

      useAppStore.getState().transcriptionStarted();
      const state = useAppStore.getState().transcription;

      expect(state.isTranscribing).toBe(true);
      expect(state.transcribedText).toBeNull();
      expect(state.error).toBeNull();
      expect(state.durationMs).toBeNull();
    });

    it("transcriptionCompleted sets text and duration, clears isTranscribing", () => {
      useAppStore.getState().transcriptionStarted();
      useAppStore.getState().transcriptionCompleted("Hello world", 1234);

      const state = useAppStore.getState().transcription;
      expect(state.isTranscribing).toBe(false);
      expect(state.transcribedText).toBe("Hello world");
      expect(state.durationMs).toBe(1234);
      expect(state.error).toBeNull();
    });

    it("transcriptionError sets error and clears isTranscribing", () => {
      useAppStore.getState().transcriptionStarted();
      useAppStore.getState().transcriptionError("Model failed to load");

      const state = useAppStore.getState().transcription;
      expect(state.isTranscribing).toBe(false);
      expect(state.error).toBe("Model failed to load");
    });

    it("transcriptionError preserves transcribedText if set", () => {
      // Edge case: error occurs after partial transcription
      useAppStore.setState({
        transcription: {
          isTranscribing: true,
          transcribedText: "Partial text",
          error: null,
          durationMs: null,
        },
      });

      useAppStore.getState().transcriptionError("Timeout");

      const state = useAppStore.getState().transcription;
      expect(state.transcribedText).toBe("Partial text");
      expect(state.error).toBe("Timeout");
    });
  });

  describe("settings", () => {
    it("setSettings updates settingsCache and marks as loaded", () => {
      const mockSettings = {
        listening: { enabled: true, wakeWord: "hey cat" },
        recording: { silenceThreshold: 0.01, device: null },
        transcription: { model: "small" },
        dictionary: { entries: [], enabled: true },
      };

      useAppStore.getState().setSettings(mockSettings as never);

      expect(useAppStore.getState().settingsCache).toEqual(mockSettings);
      expect(useAppStore.getState().isSettingsLoaded).toBe(true);
    });

    it("updateSetting updates a single setting key", () => {
      const mockSettings = {
        audio: { selectedDevice: "Default Mic" },
        shortcuts: { distinguishLeftRight: false, recordingMode: "toggle" as const },
      };

      useAppStore.getState().setSettings(mockSettings);
      useAppStore
        .getState()
        .updateSetting("audio", { selectedDevice: "USB Microphone" });

      expect(useAppStore.getState().settingsCache?.audio.selectedDevice).toBe(
        "USB Microphone"
      );
    });

    it("updateSetting does nothing when settingsCache is null", () => {
      useAppStore.getState().updateSetting("audio", {
        selectedDevice: "Test Device",
      });

      expect(useAppStore.getState().settingsCache).toBeNull();
    });
  });
});
