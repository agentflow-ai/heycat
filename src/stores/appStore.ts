import { create } from "zustand";
import type { AppSettings } from "../hooks/useSettings";

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
 */
export interface AppState {
  // Client state only - NO server state here
  overlayMode: string | null;
  settingsCache: AppSettings | null;
  isSettingsLoaded: boolean;

  // Actions
  setOverlayMode: (mode: string | null) => void;
  setSettings: (settings: AppSettings) => void;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => void;
}

export const useAppStore = create<AppState>((set) => ({
  overlayMode: null,
  settingsCache: null,
  isSettingsLoaded: false,

  setOverlayMode: (mode) => set({ overlayMode: mode }),

  setSettings: (settings) =>
    set({ settingsCache: settings, isSettingsLoaded: true }),

  updateSetting: (key, value) =>
    set((state) => ({
      settingsCache: state.settingsCache
        ? { ...state.settingsCache, [key]: value }
        : null,
    })),
}));

// Optimized selectors - components using these will only re-render
// when their specific slice changes, not on any store update
export const useOverlayMode = () => useAppStore((s) => s.overlayMode);
export const useSettingsCache = () => useAppStore((s) => s.settingsCache);
export const useIsSettingsLoaded = () => useAppStore((s) => s.isSettingsLoaded);
