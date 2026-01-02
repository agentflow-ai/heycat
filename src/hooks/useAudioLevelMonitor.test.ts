import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useAudioLevelMonitor } from "./useAudioLevelMonitor";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

// Mock listen with captured callback
const { mockListen, mockUnlisten } = vi.hoisted(() => ({
  mockListen: vi.fn(),
  mockUnlisten: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
}));

describe("useAudioLevelMonitor", () => {
  let eventCallback: ((event: { payload: number }) => void) | null = null;

  beforeEach(() => {
    vi.clearAllMocks();
    eventCallback = null;

    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockImplementation(
      async (
        _event: string,
        callback: (event: { payload: number }) => void
      ) => {
        eventCallback = callback;
        return mockUnlisten;
      }
    );
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns initial state with level 0 and not monitoring", () => {
    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: false })
    );

    expect(result.current.level).toBe(0);
    expect(result.current.isMonitoring).toBe(false);
  });

  it("starts monitoring when enabled", async () => {
    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: true })
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_audio_monitor", {
        deviceName: undefined,
      });
    });

    await waitFor(() => {
      expect(result.current.isMonitoring).toBe(true);
    });
  });

  it("passes device name to start command", async () => {
    renderHook(() =>
      useAudioLevelMonitor({ deviceName: "USB Microphone", enabled: true })
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_audio_monitor", {
        deviceName: "USB Microphone",
      });
    });
  });

  it("updates level when receiving audio-level events", async () => {
    vi.useFakeTimers();

    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: true })
    );

    // Wait for listener to be set up
    await vi.waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith(
        "audio-level",
        expect.any(Function)
      );
    });

    // Simulate audio level event
    act(() => {
      eventCallback?.({ payload: 50 });
    });

    // Advance timer to trigger the interval update
    act(() => {
      vi.advanceTimersByTime(50);
    });

    expect(result.current.level).toBe(50);
  });

  it("restarts monitoring when device changes", async () => {
    const { rerender } = renderHook(
      ({ deviceName }) => useAudioLevelMonitor({ deviceName, enabled: true }),
      { initialProps: { deviceName: null as string | null } }
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_audio_monitor", {
        deviceName: undefined,
      });
    });

    // Clear mocks and change device
    mockInvoke.mockClear();

    rerender({ deviceName: "USB Microphone" });

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("stop_audio_monitor");
    });

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_audio_monitor", {
        deviceName: "USB Microphone",
      });
    });
  });

  it("does not start monitoring when disabled", async () => {
    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: false })
    );

    // Give a bit of time for any async operations
    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(mockInvoke).not.toHaveBeenCalledWith(
      "start_audio_monitor",
      expect.anything()
    );
    expect(result.current.isMonitoring).toBe(false);
    expect(result.current.level).toBe(0);
  });

  it("stops monitoring when disabled", async () => {
    const { result, rerender } = renderHook(
      ({ enabled }) => useAudioLevelMonitor({ deviceName: null, enabled }),
      { initialProps: { enabled: true } }
    );

    await waitFor(() => {
      expect(result.current.isMonitoring).toBe(true);
    });

    mockInvoke.mockClear();

    rerender({ enabled: false });

    expect(result.current.level).toBe(0);
    expect(result.current.isMonitoring).toBe(false);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("stop_audio_monitor");
    });
  });

  it("handles start_audio_monitor error gracefully", async () => {
    mockInvoke.mockRejectedValue(new Error("Failed to start monitor"));
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: true })
    );

    await waitFor(() => {
      expect(result.current.isMonitoring).toBe(false);
    });

    expect(consoleSpy).toHaveBeenCalledWith(
      "[heycat] Failed to start audio monitor:",
      expect.any(Error)
    );

    consoleSpy.mockRestore();
  });

  it("throttles level updates to ~20fps", async () => {
    vi.useFakeTimers();

    const { result } = renderHook(() =>
      useAudioLevelMonitor({ deviceName: null, enabled: true })
    );

    // Wait for listener to be set up
    await vi.waitFor(() => {
      expect(mockListen).toHaveBeenCalled();
    });

    // Simulate multiple rapid events
    act(() => {
      eventCallback?.({ payload: 10 });
      eventCallback?.({ payload: 20 });
      eventCallback?.({ payload: 30 });
    });

    // Level should not update immediately (before interval)
    expect(result.current.level).toBe(0);

    // Advance timer to trigger update
    act(() => {
      vi.advanceTimersByTime(50);
    });

    // Should have latest value
    expect(result.current.level).toBe(30);
  });
});
