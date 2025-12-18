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
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("user can start and stop recording, receiving metadata on completion", async () => {
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

    // User starts recording
    await act(async () => {
      await result.current.startRecording();
    });

    expect(mockInvoke).toHaveBeenCalledWith("start_recording", {
      deviceName: undefined,
    });

    // Backend confirms recording started
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);

    // User stops recording
    await act(async () => {
      await result.current.stopRecording();
    });

    expect(mockInvoke).toHaveBeenCalledWith("stop_recording");

    // Backend confirms recording stopped with metadata
    act(() => {
      stoppedCallback!({ payload: { metadata } });
    });

    expect(result.current.isRecording).toBe(false);
    expect(result.current.lastRecording).toEqual(metadata);
    expect(result.current.error).toBeNull();
  });

  it("user sees error when recording fails to start", async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_recording_state") {
        return Promise.resolve({ state: "Idle" });
      }
      if (command === "start_recording") {
        return Promise.reject(new Error("Microphone not found"));
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useRecording());

    await act(async () => {
      await result.current.startRecording();
    });

    expect(result.current.error).toBe("Microphone not found");
    expect(result.current.isRecording).toBe(false);
  });

  it("user sees error when backend emits recording_error", async () => {
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

  it("state reflects backend events even without explicit user action", async () => {
    // This covers scenarios where recording state changes due to external factors
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

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(stoppedCallback).not.toBeNull();
    });

    // Backend emits started event (e.g., from hotkey trigger)
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);

    // Backend emits stopped event
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
  });

  it("user cancels recording via double-tap-escape, state shows cancellation", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let cancelledCallback: ((event: { payload: { reason: string; timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        } else if (eventName === "recording_cancelled") {
          cancelledCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(cancelledCallback).not.toBeNull();
    });

    // Recording starts
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);
    expect(result.current.wasCancelled).toBe(false);

    // User cancels via double-tap-escape
    act(() => {
      cancelledCallback!({ payload: { reason: "double-tap-escape", timestamp: "2025-01-01T12:00:05Z" } });
    });

    expect(result.current.isRecording).toBe(false);
    expect(result.current.wasCancelled).toBe(true);
    expect(result.current.cancelReason).toBe("double-tap-escape");
    expect(result.current.error).toBeNull();
  });

  it("cancelled state resets when new recording starts", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let cancelledCallback: ((event: { payload: { reason: string; timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "recording_started") {
          startedCallback = callback;
        } else if (eventName === "recording_cancelled") {
          cancelledCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useRecording());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(cancelledCallback).not.toBeNull();
    });

    // First recording starts and gets cancelled
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    act(() => {
      cancelledCallback!({ payload: { reason: "double-tap-escape", timestamp: "2025-01-01T12:00:05Z" } });
    });
    expect(result.current.wasCancelled).toBe(true);
    expect(result.current.cancelReason).toBe("double-tap-escape");

    // New recording starts - cancelled state should reset
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:01:00Z" } });
    });
    expect(result.current.isRecording).toBe(true);
    expect(result.current.wasCancelled).toBe(false);
    expect(result.current.cancelReason).toBeNull();
  });
});
