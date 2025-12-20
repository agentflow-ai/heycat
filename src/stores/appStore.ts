import { create } from "zustand";
import type { AppSettings } from "../hooks/useSettings";

/**
 * Transcription state for the current/most recent transcription.
 * This is transient UI state updated via events.
 */
export interface TranscriptionState {
  isTranscribing: boolean;
  transcribedText: string | null;
  error: string | null;
  durationMs: number | null;
}

/**
 * Listening state for transient UI events.
 * The main listening status (isListening, isMicAvailable) comes from Tanstack Query.
 * This only holds transient state that auto-resets.
 */
export interface ListeningUIState {
  isWakeWordDetected: boolean;
}

/**
 * Global app state managed by Zustand.
 *
 * IMPORTANT: This store holds CLIENT state only. Server state (recordings,
 * recording state, model status) belongs in Tanstack Query, not here.
 *
 * State types:
 * - overlayMode: Current overlay visibility state (e.g., "recording", "commands")
 * - settingsCache: In-memory cache of settings from Tauri Store
 * - isSettingsLoaded: Hydration flag indicating settings have been loaded
 * - transcription: Current transcription state (updated via events)
 * - listening: Transient listening UI state (wake word detection)
 */
export interface AppState {
  // Client state only - NO server state here
  overlayMode: string | null;
  settingsCache: AppSettings | null;
  isSettingsLoaded: boolean;
  transcription: TranscriptionState;
  listening: ListeningUIState;

  // Actions
  setOverlayMode: (mode: string | null) => void;
  setSettings: (settings: AppSettings) => void;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => void;
  transcriptionStarted: () => void;
  transcriptionCompleted: (text: string, durationMs: number) => void;
  transcriptionError: (error: string) => void;
  wakeWordDetected: () => void;
  clearWakeWord: () => void;
}

const initialTranscriptionState: TranscriptionState = {
  isTranscribing: false,
  transcribedText: null,
  error: null,
  durationMs: null,
};

const initialListeningUIState: ListeningUIState = {
  isWakeWordDetected: false,
};

export const useAppStore = create<AppState>((set) => ({
  overlayMode: null,
  settingsCache: null,
  isSettingsLoaded: false,
  transcription: initialTranscriptionState,
  listening: initialListeningUIState,

  setOverlayMode: (mode) => set({ overlayMode: mode }),

  setSettings: (settings) =>
    set({ settingsCache: settings, isSettingsLoaded: true }),

  updateSetting: (key, value) =>
    set((state) => ({
      settingsCache: state.settingsCache
        ? { ...state.settingsCache, [key]: value }
        : null,
    })),

  transcriptionStarted: () =>
    set({
      transcription: {
        isTranscribing: true,
        transcribedText: null,
        error: null,
        durationMs: null,
      },
    }),

  transcriptionCompleted: (text, durationMs) =>
    set({
      transcription: {
        isTranscribing: false,
        transcribedText: text,
        error: null,
        durationMs,
      },
    }),

  transcriptionError: (error) =>
    set((state) => ({
      transcription: {
        ...state.transcription,
        isTranscribing: false,
        error,
      },
    })),

  wakeWordDetected: () =>
    set({ listening: { isWakeWordDetected: true } }),

  clearWakeWord: () =>
    set({ listening: { isWakeWordDetected: false } }),
}));

// Optimized selectors - components using these will only re-render
// when their specific slice changes, not on any store update
export const useOverlayMode = () => useAppStore((s) => s.overlayMode);
export const useSettingsCache = () => useAppStore((s) => s.settingsCache);
export const useIsSettingsLoaded = () => useAppStore((s) => s.isSettingsLoaded);
export const useTranscriptionState = () => useAppStore((s) => s.transcription);
export const useListeningUIState = () => useAppStore((s) => s.listening);
