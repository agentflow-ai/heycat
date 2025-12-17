import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useSettings } from "./useSettings";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
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
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with default settings", async () => {
    const { result } = renderHook(() => useSettings());

    // Initially loading
    expect(result.current.isLoading).toBe(true);
    expect(result.current.settings.listening.enabled).toBe(false);
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(false);
    expect(result.current.settings.audio.selectedDevice).toBeNull();

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });

  it("loads persisted settings from store", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "listening.enabled") return Promise.resolve(true);
      if (key === "listening.autoStartOnLaunch") return Promise.resolve(true);
      if (key === "audio.selectedDevice") return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.listening.enabled).toBe(true);
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(true);
    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");
  });

  it("updateListeningEnabled saves to store and updates state", async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateListeningEnabled(true);
    });

    expect(mockStore.set).toHaveBeenCalledWith("listening.enabled", true);
    expect(result.current.settings.listening.enabled).toBe(true);
  });

  it("updateAutoStartListening saves to store and updates state", async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateAutoStartListening(true);
    });

    expect(mockStore.set).toHaveBeenCalledWith(
      "listening.autoStartOnLaunch",
      true
    );
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(true);
  });

  it("handles store load error", async () => {
    const { load } = await import("@tauri-apps/plugin-store");
    vi.mocked(load).mockRejectedValueOnce(new Error("Store failed"));

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBe("Store failed");
  });

  it("handles store set error", async () => {
    mockStore.set.mockRejectedValueOnce(new Error("Write failed"));
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateListeningEnabled(true);
    });

    expect(result.current.error).toBe("Write failed");
  });

  it("clears error on successful update", async () => {
    mockStore.set
      .mockRejectedValueOnce(new Error("First error"))
      .mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateListeningEnabled(true);
    });
    expect(result.current.error).toBe("First error");

    await act(async () => {
      await result.current.updateAutoStartListening(true);
    });
    expect(result.current.error).toBeNull();
  });

  it("returns stable function references", async () => {
    const { result, rerender } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const updateEnabled1 = result.current.updateListeningEnabled;
    const updateAutoStart1 = result.current.updateAutoStartListening;

    rerender();

    // Functions should be stable (memoized with useCallback)
    expect(result.current.updateListeningEnabled).toBe(updateEnabled1);
    expect(result.current.updateAutoStartListening).toBe(updateAutoStart1);
  });

  it("uses default values when store returns undefined", async () => {
    mockStore.get.mockResolvedValue(undefined);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.listening.enabled).toBe(false);
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(false);
    expect(result.current.settings.audio.selectedDevice).toBeNull();
  });

  it("updateAudioDevice saves to store and updates state", async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateAudioDevice("USB Microphone");
    });

    expect(mockStore.set).toHaveBeenCalledWith(
      "audio.selectedDevice",
      "USB Microphone"
    );
    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");
  });

  it("updateAudioDevice can clear selection with null", async () => {
    // Start with a device selected
    mockStore.get.mockImplementation((key: string) => {
      if (key === "audio.selectedDevice")
        return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");

    await act(async () => {
      await result.current.updateAudioDevice(null);
    });

    expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", null);
    expect(result.current.settings.audio.selectedDevice).toBeNull();
  });

  it("updateAudioDevice handles store set error", async () => {
    mockStore.set.mockRejectedValueOnce(new Error("Audio write failed"));
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.updateAudioDevice("USB Microphone");
    });

    expect(result.current.error).toBe("Audio write failed");
  });

  it("updateAudioDevice returns stable function reference", async () => {
    const { result, rerender } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const updateAudioDevice1 = result.current.updateAudioDevice;

    rerender();

    expect(result.current.updateAudioDevice).toBe(updateAudioDevice1);
  });

  it("does nothing when updateListeningEnabled called before store loads", async () => {
    // Delay store load indefinitely
    const { load } = await import("@tauri-apps/plugin-store");
    vi.mocked(load).mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useSettings());

    // Store is still loading, try to update
    await act(async () => {
      await result.current.updateListeningEnabled(true);
    });

    // Should not throw, store.set should not be called
    expect(mockStore.set).not.toHaveBeenCalled();
  });

  it("does nothing when updateAudioDevice called before store loads", async () => {
    // Delay store load indefinitely
    const { load } = await import("@tauri-apps/plugin-store");
    vi.mocked(load).mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useSettings());

    // Store is still loading, try to update
    await act(async () => {
      await result.current.updateAudioDevice("USB Microphone");
    });

    // Should not throw, store.set should not be called
    expect(mockStore.set).not.toHaveBeenCalled();
  });
});
