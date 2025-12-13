import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useMultiModelStatus } from "./useMultiModelStatus";

// Mock invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock listen
const mockListeners: Map<string, ((event: { payload: unknown }) => void)[]> = new Map();
const mockListen = vi.fn().mockImplementation((eventName: string, callback: (event: { payload: unknown }) => void) => {
  const listeners = mockListeners.get(eventName) || [];
  listeners.push(callback);
  mockListeners.set(eventName, listeners);
  return Promise.resolve(() => {
    const currentListeners = mockListeners.get(eventName) || [];
    const index = currentListeners.indexOf(callback);
    if (index > -1) {
      currentListeners.splice(index, 1);
    }
  });
});

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

function emitEvent(eventName: string, payload: unknown) {
  const listeners = mockListeners.get(eventName) || [];
  listeners.forEach((cb) => cb({ payload }));
}

describe("useMultiModelStatus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListeners.clear();

    // Default mocks
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(false);
      }
      return Promise.resolve();
    });
  });

  it("initializes with both models as unavailable", async () => {
    const { result } = renderHook(() => useMultiModelStatus());

    expect(result.current.models.tdt.isAvailable).toBe(false);
    expect(result.current.models.eou.isAvailable).toBe(false);
  });

  it("checks both model statuses on mount", async () => {
    renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", { modelType: "ParakeetTDT" });
      expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", { modelType: "ParakeetEOU" });
    });
  });

  it("updates model availability when status check returns true", async () => {
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(args?.modelType === "ParakeetTDT");
      }
      return Promise.resolve();
    });

    const { result } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(result.current.models.tdt.isAvailable).toBe(true);
      expect(result.current.models.tdt.downloadState).toBe("completed");
      expect(result.current.models.eou.isAvailable).toBe(false);
    });
  });

  it("downloadModel triggers download_model invoke with correct modelType", async () => {
    const { result } = renderHook(() => useMultiModelStatus());

    await act(async () => {
      await result.current.downloadModel("tdt");
    });

    expect(mockInvoke).toHaveBeenCalledWith("download_model", { modelType: "tdt" });
  });

  it("sets downloadState to 'downloading' when download starts", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "download_model") {
        return new Promise(() => {}); // Never resolves
      }
      return Promise.resolve(false);
    });

    const { result } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(result.current.models.tdt.downloadState).toBe("idle");
    });

    act(() => {
      result.current.downloadModel("tdt");
    });

    await waitFor(() => {
      expect(result.current.models.tdt.downloadState).toBe("downloading");
    });
  });

  it("updates progress when model_file_download_progress event is received", async () => {
    const { result } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith("model_file_download_progress", expect.any(Function));
    });

    act(() => {
      emitEvent("model_file_download_progress", {
        model_type: "tdt",
        file_name: "model.bin",
        percent: 50,
        bytes_downloaded: 500,
        total_bytes: 1000,
      });
    });

    expect(result.current.models.tdt.progress).toBe(50);
  });

  it("sets downloadState to 'completed' when model_download_completed event is received", async () => {
    const { result } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith("model_download_completed", expect.any(Function));
    });

    act(() => {
      emitEvent("model_download_completed", {
        model_type: "eou",
        model_path: "/path/to/model",
      });
    });

    expect(result.current.models.eou.isAvailable).toBe(true);
    expect(result.current.models.eou.downloadState).toBe("completed");
    expect(result.current.models.eou.progress).toBe(100);
  });

  it("sets downloadState to 'error' when download fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "download_model") {
        return Promise.reject(new Error("Download failed"));
      }
      return Promise.resolve(false);
    });

    const { result } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(result.current.models.tdt.downloadState).toBe("idle");
    });

    await act(async () => {
      await result.current.downloadModel("tdt");
    });

    expect(result.current.models.tdt.downloadState).toBe("error");
    expect(result.current.models.tdt.error).toBe("Download failed");
  });

  it("refreshStatus updates both model availabilities", async () => {
    const { result } = renderHook(() => useMultiModelStatus());

    // Initial check shows both unavailable
    await waitFor(() => {
      expect(result.current.models.tdt.isAvailable).toBe(false);
    });

    // Now model becomes available
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(true);
      }
      return Promise.resolve();
    });

    await act(async () => {
      await result.current.refreshStatus();
    });

    expect(result.current.models.tdt.isAvailable).toBe(true);
    expect(result.current.models.eou.isAvailable).toBe(true);
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useMultiModelStatus());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalled();
    });

    const initialListenerCount = mockListeners.get("model_file_download_progress")?.length || 0;
    expect(initialListenerCount).toBeGreaterThan(0);

    unmount();

    // Listeners should be cleaned up
    const finalListenerCount = mockListeners.get("model_file_download_progress")?.length || 0;
    expect(finalListenerCount).toBe(0);
  });
});
