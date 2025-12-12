import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useTranscription } from "./useTranscription";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("useTranscription", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default listen mock returns an unlisten function
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with default state", () => {
    const { result } = renderHook(() => useTranscription());

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });

  it("sets up event listeners on mount", async () => {
    renderHook(() => useTranscription());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(3);
    });

    expect(mockListen).toHaveBeenCalledWith(
      "transcription_started",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "transcription_completed",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "transcription_error",
      expect.any(Function)
    );
  });

  it("updates state when transcription_started event fires", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.error).toBeNull();
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });

  it("updates state when transcription_completed event fires", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let completedCallback: ((event: {
      payload: { text: string; duration_ms: number };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(completedCallback).not.toBeNull();
    });

    // Start transcription
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);

    // Complete transcription
    act(() => {
      completedCallback!({
        payload: { text: "Hello, world!", duration_ms: 1234 },
      });
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBe("Hello, world!");
    expect(result.current.durationMs).toBe(1234);
    expect(result.current.error).toBeNull();
  });

  it("updates state when transcription_error event fires", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let errorCallback: ((event: { payload: { error: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(errorCallback).not.toBeNull();
    });

    // Start transcription
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);

    // Error during transcription
    act(() => {
      errorCallback!({ payload: { error: "Model not loaded" } });
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.error).toBe("Model not loaded");
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(3);
    });

    unmount();

    // Each listener's unlisten function should be called
    expect(mockUnlisten).toHaveBeenCalledTimes(3);
  });

  it("clears previous state when new transcription starts", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let completedCallback: ((event: {
      payload: { text: string; duration_ms: number };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(completedCallback).not.toBeNull();
    });

    // Complete first transcription
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    act(() => {
      completedCallback!({
        payload: { text: "First transcription", duration_ms: 1000 },
      });
    });
    expect(result.current.transcribedText).toBe("First transcription");
    expect(result.current.durationMs).toBe(1000);

    // Start second transcription - should clear previous state
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:01:00Z" } });
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.durationMs).toBeNull();
    expect(result.current.error).toBeNull();
  });
});
