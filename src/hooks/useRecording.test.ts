import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useRecording, RecordingMetadata } from "./useRecording";

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

describe("useRecording", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default listen mock returns an unlisten function
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with isRecording: false and no error", () => {
    const { result } = renderHook(() => useRecording());

    expect(result.current.isRecording).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.lastRecording).toBeNull();
  });

  it("startRecording() calls invoke and state updates via event", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    mockInvoke.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    await act(async () => {
      await result.current.startRecording();
    });

    expect(mockInvoke).toHaveBeenCalledWith("start_recording");
    // State doesn't update immediately - needs event
    expect(result.current.isRecording).toBe(false);

    // Simulate backend emitting the event
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });

    expect(result.current.isRecording).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it("stopRecording() calls invoke and state updates via event", async () => {
    const metadata: RecordingMetadata = {
      duration_secs: 5.5,
      file_path: "/tmp/test.wav",
      sample_count: 88200,
    };

    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let stoppedCallback: ((event: { payload: { metadata: RecordingMetadata } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        } else if (eventName === "recording_stopped") {
          stoppedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(stoppedCallback).not.toBeNull();
    });

    // Start recording
    await act(async () => {
      await result.current.startRecording();
    });
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);

    // Stop recording
    await act(async () => {
      await result.current.stopRecording();
    });

    expect(mockInvoke).toHaveBeenCalledWith("stop_recording");

    // Simulate backend emitting the stopped event
    act(() => {
      stoppedCallback!({ payload: { metadata } });
    });

    expect(result.current.isRecording).toBe(false);
    expect(result.current.lastRecording).toEqual(metadata);
    expect(result.current.error).toBeNull();
  });

  it("sets error state when startRecording fails", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Microphone not found"));
    const { result } = renderHook(() => useRecording());

    await act(async () => {
      await result.current.startRecording();
    });

    expect(result.current.error).toBe("Microphone not found");
    expect(result.current.isRecording).toBe(false);
  });

  it("sets error state when stopRecording fails", async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // start succeeds
    mockInvoke.mockRejectedValueOnce("Not recording"); // stop fails (string error)

    const { result } = renderHook(() => useRecording());

    await act(async () => {
      await result.current.startRecording();
    });

    await act(async () => {
      await result.current.stopRecording();
    });

    expect(result.current.error).toBe("Not recording");
  });

  it("sets up event listeners on mount", async () => {
    renderHook(() => useRecording());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(3);
    });

    expect(mockListen).toHaveBeenCalledWith(
      "recording_started",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "recording_stopped",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "recording_error",
      expect.any(Function)
    );
  });

  it("event listener updates state when backend emits recording_started", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });

    expect(result.current.isRecording).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it("event listener updates state when backend emits recording_stopped", async () => {
    let stoppedCallback: ((event: {
      payload: { metadata: RecordingMetadata };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_stopped") {
          stoppedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useRecording());

    // First set isRecording to true via startRecording
    mockInvoke.mockResolvedValueOnce(undefined);
    await act(async () => {
      await result.current.startRecording();
    });

    await waitFor(() => {
      expect(stoppedCallback).not.toBeNull();
    });

    const metadata: RecordingMetadata = {
      duration_secs: 3.0,
      file_path: "/tmp/recording.wav",
      sample_count: 48000,
    };

    act(() => {
      stoppedCallback!({ payload: { metadata } });
    });

    expect(result.current.isRecording).toBe(false);
    expect(result.current.lastRecording).toEqual(metadata);
    expect(result.current.error).toBeNull();
  });

  it("event listener updates error state when backend emits recording_error", async () => {
    let errorCallback: ((event: { payload: { message: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    act(() => {
      errorCallback!({ payload: { message: "Audio device disconnected" } });
    });

    expect(result.current.error).toBe("Audio device disconnected");
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(3);
    });

    unmount();

    // Each listener's unlisten function should be called
    expect(mockUnlisten).toHaveBeenCalledTimes(3);
  });

  it("returns stable function references", async () => {
    const { result, rerender } = renderHook(() => useRecording());

    const startRecording1 = result.current.startRecording;
    const stopRecording1 = result.current.stopRecording;

    rerender();

    expect(result.current.startRecording).toBe(startRecording1);
    expect(result.current.stopRecording).toBe(stopRecording1);
  });

  it("clears error on successful startRecording", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    // First set an error
    mockInvoke.mockRejectedValueOnce(new Error("Initial error"));
    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    await act(async () => {
      await result.current.startRecording();
    });
    expect(result.current.error).toBe("Initial error");

    // Now succeed - error clears immediately on calling startRecording
    mockInvoke.mockResolvedValueOnce(undefined);
    await act(async () => {
      await result.current.startRecording();
    });

    expect(result.current.error).toBeNull();

    // State updates via event
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);
  });

  it("clears error on successful stopRecording", async () => {
    const metadata: RecordingMetadata = {
      duration_secs: 1.0,
      file_path: "/tmp/test.wav",
      sample_count: 16000,
    };

    // Start recording
    mockInvoke.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useRecording());

    await act(async () => {
      await result.current.startRecording();
    });

    // Fail stop once to set error
    mockInvoke.mockRejectedValueOnce(new Error("Stop error"));
    await act(async () => {
      await result.current.stopRecording();
    });
    expect(result.current.error).toBe("Stop error");

    // Now succeed
    mockInvoke.mockResolvedValueOnce(metadata);
    await act(async () => {
      await result.current.stopRecording();
    });

    expect(result.current.error).toBeNull();
  });
});
