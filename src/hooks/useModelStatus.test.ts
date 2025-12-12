import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useModelStatus } from "./useModelStatus";

// Mock Tauri APIs
const mockInvoke = vi.fn();
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("useModelStatus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: model not available, listen returns unlisten function
    mockInvoke.mockResolvedValue(false);
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with model not available and idle state", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    });

    expect(result.current.isModelAvailable).toBe(false);
    expect(result.current.downloadState).toBe("idle");
    expect(result.current.error).toBeNull();
  });

  it("checks model status on mount", async () => {
    mockInvoke.mockResolvedValueOnce(true);

    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(result.current.isModelAvailable).toBe(true);
    });

    expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    expect(result.current.downloadState).toBe("completed");
  });

  it("downloadModel() sets downloading state and calls invoke", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    });

    mockInvoke.mockResolvedValueOnce("/path/to/model.bin");

    await act(async () => {
      await result.current.downloadModel();
    });

    expect(mockInvoke).toHaveBeenCalledWith("download_model");
  });

  it("sets error state when download fails", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    });

    mockInvoke.mockRejectedValueOnce(new Error("Network error"));

    await act(async () => {
      await result.current.downloadModel();
    });

    expect(result.current.downloadState).toBe("error");
    expect(result.current.error).toBe("Network error");
  });

  it("handles string error from download failure", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    });

    mockInvoke.mockRejectedValueOnce("Connection refused");

    await act(async () => {
      await result.current.downloadModel();
    });

    expect(result.current.downloadState).toBe("error");
    expect(result.current.error).toBe("Connection refused");
  });

  it("sets up event listener for model_download_completed", async () => {
    renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "model_download_completed",
        expect.any(Function)
      );
    });
  });

  it("updates state when model_download_completed event fires", async () => {
    let completedCallback: ((event: {
      payload: { model_path: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "model_download_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(completedCallback).not.toBeNull();
    });

    // Start download to set downloading state
    mockInvoke.mockResolvedValueOnce("/path/to/model.bin");
    await act(async () => {
      await result.current.downloadModel();
    });

    // Simulate backend emitting the event
    act(() => {
      completedCallback!({ payload: { model_path: "/path/to/model.bin" } });
    });

    expect(result.current.isModelAvailable).toBe(true);
    expect(result.current.downloadState).toBe("completed");
    expect(result.current.error).toBeNull();
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(1);
    });

    unmount();

    expect(mockUnlisten).toHaveBeenCalledTimes(1);
  });

  it("refreshStatus() updates model availability", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_model_status");
    });

    // Model wasn't available initially
    expect(result.current.isModelAvailable).toBe(false);

    // Now model is available
    mockInvoke.mockResolvedValueOnce(true);

    await act(async () => {
      await result.current.refreshStatus();
    });

    expect(result.current.isModelAvailable).toBe(true);
    expect(result.current.downloadState).toBe("completed");
  });

  it("sets error when check_model_status fails", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Data dir not found"));

    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(result.current.error).toBe("Data dir not found");
    });
  });

  it("returns stable function references", async () => {
    const { result, rerender } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled();
    });

    const downloadModel1 = result.current.downloadModel;
    const refreshStatus1 = result.current.refreshStatus;

    rerender();

    expect(result.current.downloadModel).toBe(downloadModel1);
    expect(result.current.refreshStatus).toBe(refreshStatus1);
  });

  it("clears error when download starts", async () => {
    const { result } = renderHook(() => useModelStatus());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled();
    });

    // First fail to set error
    mockInvoke.mockRejectedValueOnce(new Error("First error"));
    await act(async () => {
      await result.current.downloadModel();
    });
    expect(result.current.error).toBe("First error");

    // Start new download - error should clear immediately
    mockInvoke.mockResolvedValueOnce("/path/to/model.bin");
    await act(async () => {
      await result.current.downloadModel();
    });

    // Error clears when download starts (before it fails)
    // Since this succeeded, error should still be null
    expect(result.current.error).toBeNull();
  });
});
