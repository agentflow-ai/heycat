import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useAutoStartListening } from "./useAutoStartListening";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore, mockLoad, mockInvoke } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
  },
  mockLoad: vi.fn(),
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: mockLoad,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

describe("useAutoStartListening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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
      expect(mockLoad).toHaveBeenCalledWith("settings.json", {
        autoSave: false,
      });
    });
  });

  it("calls enable_listening when autoStartOnLaunch is true", async () => {
    mockStore.get.mockResolvedValue(true);

    renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockStore.get).toHaveBeenCalledWith("listening.autoStartOnLaunch");
      expect(mockInvoke).toHaveBeenCalledWith("enable_listening");
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

  it("only runs once even when re-rendered", async () => {
    mockStore.get.mockResolvedValue(true);

    const { rerender } = renderHook(() => useAutoStartListening());

    await vi.waitFor(() => {
      expect(mockLoad).toHaveBeenCalledTimes(1);
    });

    // Re-render multiple times
    rerender();
    rerender();
    rerender();

    // Should still only have called load once
    expect(mockLoad).toHaveBeenCalledTimes(1);
  });

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
