import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useMultiModelStatus } from "./useMultiModelStatus";

// Mock invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock listen
const mockListeners: Map<string, ((event: { payload: unknown }) => void)[]> =
  new Map();
const mockListen = vi
  .fn()
  .mockImplementation(
    (eventName: string, callback: (event: { payload: unknown }) => void) => {
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
    }
  );

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

function emitEvent(eventName: string, payload: unknown) {
  const listeners = mockListeners.get(eventName) || [];
  listeners.forEach((cb) => cb({ payload }));
}

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      children
    );
  };
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

  it("initializes with model as unavailable", async () => {
    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    expect(result.current.models.isAvailable).toBe(false);
  });

  it("checks model status on mount", async () => {
    renderHook(() => useMultiModelStatus(), { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", {
        modelType: "tdt",
      });
    });
  });

  it("updates model availability when status check returns true", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(true);
      }
      return Promise.resolve();
    });

    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.models.isAvailable).toBe(true);
      expect(result.current.models.downloadState).toBe("completed");
    });
  });

  it("downloadModel triggers download_model invoke with correct modelType", async () => {
    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.downloadModel("tdt");
    });

    expect(mockInvoke).toHaveBeenCalledWith("download_model", {
      modelType: "tdt",
    });
  });

  it("sets downloadState to 'downloading' when download starts", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "download_model") {
        return new Promise(() => {}); // Never resolves
      }
      return Promise.resolve(false);
    });

    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.models.downloadState).toBe("idle");
    });

    act(() => {
      result.current.downloadModel("tdt");
    });

    await waitFor(() => {
      expect(result.current.models.downloadState).toBe("downloading");
    });
  });

  it("updates progress when model_file_download_progress event is received", async () => {
    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "model_file_download_progress",
        expect.any(Function)
      );
    });

    act(() => {
      emitEvent("model_file_download_progress", {
        modelType: "tdt",
        fileName: "model.bin",
        percent: 50,
        bytesDownloaded: 500,
        totalBytes: 1000,
      });
    });

    expect(result.current.models.progress).toBe(50);
  });

  it("sets downloadState to 'completed' when model_download_completed event is received", async () => {
    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "model_download_completed",
        expect.any(Function)
      );
    });

    act(() => {
      emitEvent("model_download_completed", {
        modelType: "tdt",
        modelPath: "/path/to/model",
      });
    });

    expect(result.current.models.downloadState).toBe("completed");
    expect(result.current.models.progress).toBe(100);
  });

  it("sets downloadState to 'error' when download fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "download_model") {
        return Promise.reject(new Error("Download failed"));
      }
      return Promise.resolve(false);
    });

    const { result } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.models.downloadState).toBe("idle");
    });

    await act(async () => {
      try {
        await result.current.downloadModel("tdt");
      } catch {
        // Expected to throw
      }
    });

    expect(result.current.models.downloadState).toBe("error");
    expect(result.current.models.error).toBe("Download failed");
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useMultiModelStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalled();
    });

    const initialListenerCount =
      mockListeners.get("model_file_download_progress")?.length || 0;
    expect(initialListenerCount).toBeGreaterThan(0);

    unmount();

    // Listeners should be cleaned up
    const finalListenerCount =
      mockListeners.get("model_file_download_progress")?.length || 0;
    expect(finalListenerCount).toBe(0);
  });
});
