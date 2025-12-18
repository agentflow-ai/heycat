import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useListening } from "./useListening";

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

describe("useListening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("user can enable and disable listening mode", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let stoppedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        } else if (eventName === "listening_stopped") {
          stoppedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(stoppedCallback).not.toBeNull();
    });

    // User enables listening
    await act(async () => {
      await result.current.enableListening();
    });

    expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
      deviceName: undefined,
    });

    // Backend confirms listening started
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isListening).toBe(true);

    // User disables listening
    await act(async () => {
      await result.current.disableListening();
    });

    expect(mockInvoke).toHaveBeenCalledWith("disable_listening");

    // Backend confirms listening stopped
    act(() => {
      stoppedCallback!({ payload: { timestamp: "2025-01-01T12:00:01Z" } });
    });

    expect(result.current.isListening).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("user sees wake word detection indicator temporarily", async () => {
    vi.useFakeTimers();

    let wakeWordCallback: ((event: {
      payload: { confidence: number; transcription: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "wake_word_detected") {
          wakeWordCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    await act(async () => {
      await vi.runAllTimersAsync();
    });

    expect(wakeWordCallback).not.toBeNull();
    expect(result.current.isWakeWordDetected).toBe(false);

    // Wake word detected
    act(() => {
      wakeWordCallback!({
        payload: {
          confidence: 0.95,
          transcription: "hey cat",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });

    expect(result.current.isWakeWordDetected).toBe(true);

    // After timeout, indicator resets
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current.isWakeWordDetected).toBe(false);

    vi.useRealTimers();
  });

  it("user sees error and mic unavailable when microphone disconnects", async () => {
    let unavailableCallback: ((event: {
      payload: { reason: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_unavailable") {
          unavailableCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(unavailableCallback).not.toBeNull();
    });

    expect(result.current.isMicAvailable).toBe(true);

    // Mic becomes unavailable
    act(() => {
      unavailableCallback!({
        payload: {
          reason: "Microphone disconnected",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });

    expect(result.current.isMicAvailable).toBe(false);
    expect(result.current.isListening).toBe(false);
    expect(result.current.error).toBe("Microphone disconnected");
  });

  it("user sees error when enabling listening fails", async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_listening_status") {
        return Promise.resolve({ enabled: false, active: false, micAvailable: true });
      }
      if (command === "enable_listening") {
        return Promise.reject(new Error("Cannot enable while recording"));
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useListening());

    await act(async () => {
      await result.current.enableListening();
    });

    expect(result.current.error).toBe("Cannot enable while recording");
    expect(result.current.isListening).toBe(false);
  });

  it("mic availability recovers when listening successfully starts", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let unavailableCallback: ((event: {
      payload: { reason: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        } else if (eventName === "listening_unavailable") {
          unavailableCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(unavailableCallback).not.toBeNull();
    });

    // First make mic unavailable
    act(() => {
      unavailableCallback!({
        payload: {
          reason: "Microphone disconnected",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });
    expect(result.current.isMicAvailable).toBe(false);

    // Mic reconnected and listening starts
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:01Z" } });
    });

    expect(result.current.isMicAvailable).toBe(true);
    expect(result.current.isListening).toBe(true);
  });
});
