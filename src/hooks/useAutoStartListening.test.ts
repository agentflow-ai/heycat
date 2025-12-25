import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useAutoStartListening } from "./useAutoStartListening";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore, mockLoad, mockInvoke, mockGetSettingsFile } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
  },
  mockLoad: vi.fn(),
  mockInvoke: vi.fn(),
  mockGetSettingsFile: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: mockLoad,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("../lib/settingsFile", () => ({
  getSettingsFile: mockGetSettingsFile,
}));

describe("useAutoStartListening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetSettingsFile.mockResolvedValue("settings.json");
    mockLoad.mockResolvedValue(mockStore);
    mockInvoke.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("loads settings from store on mount", async () => {
    mockStore.get.mockResolvedValue(false);

    renderHook(() => useAutoStartListening());

    // Allow async effect to run
    await vi.waitFor(() => {
      expect(mockLoad).toHaveBeenCalledWith("settings.json");
    });
  });

  it("calls enable_listening when autoStartOnLaunch is true", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "listening.autoStartOnLaunch") return Promise.resolve(true);
      if (key === "audio.selectedDevice") return Promise.resolve(null);
      return Promise.resolve(undefined);
    });

    renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockStore.get).toHaveBeenCalledWith("listening.autoStartOnLaunch");
      expect(mockStore.get).toHaveBeenCalledWith("audio.selectedDevice");
      expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
        deviceName: undefined,
      });
    });
  });

  it("calls enable_listening with selected device when set", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "listening.autoStartOnLaunch") return Promise.resolve(true);
      if (key === "audio.selectedDevice")
        return Promise.resolve("My Headset Microphone");
      return Promise.resolve(undefined);
    });

    renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
        deviceName: "My Headset Microphone",
      });
    });
  });

  it("does not call enable_listening when autoStartOnLaunch is false", async () => {
    mockStore.get.mockResolvedValue(false);

    renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockStore.get).toHaveBeenCalledWith("listening.autoStartOnLaunch");
    });

    // Give some time for potential invoke call
    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not call enable_listening when autoStartOnLaunch is undefined", async () => {
    mockStore.get.mockResolvedValue(undefined);

    renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockStore.get).toHaveBeenCalledWith("listening.autoStartOnLaunch");
    });

    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  // Re-render test removed per TESTING.md:
  // - Testing that useEffect runs once is testing React's dependency array implementation
  // - This verifies React works correctly, not our code's behavior

  it("silently handles store load errors", async () => {
    mockLoad.mockRejectedValue(new Error("Store failed"));

    // Should not throw
    expect(() => {
      renderHook(() => useAutoStartListening());
    }).not.toThrow();

    await vi.waitFor(() => {
      expect(mockLoad).toHaveBeenCalled();
    });

    // Should not call invoke if store failed
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("silently handles invoke errors", async () => {
    mockStore.get.mockResolvedValue(true);
    mockInvoke.mockRejectedValue(new Error("Invoke failed"));

    // Should not throw
    expect(() => {
      renderHook(() => useAutoStartListening());
    }).not.toThrow();

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled();
    });
  });
});
