import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useAudioDevices } from "./useAudioDevices";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

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

describe("useAudioDevices", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("basic functionality", () => {
    it("returns loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));
      const { result } = renderHook(
        () => useAudioDevices({ autoRefresh: false }),
        { wrapper: createWrapper() }
      );

      expect(result.current.isLoading).toBe(true);
      expect(result.current.devices).toEqual([]);
      expect(result.current.error).toBeNull();
    });

    it("fetches devices on mount", async () => {
      const mockDevices = [
        { name: "Built-in Microphone", isDefault: true },
        { name: "USB Microphone", isDefault: false },
      ];
      mockInvoke.mockResolvedValue(mockDevices);

      const { result } = renderHook(
        () => useAudioDevices({ autoRefresh: false }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith("list_audio_devices");
      expect(result.current.devices).toEqual(mockDevices);
      expect(result.current.error).toBeNull();
    });

    it("handles fetch error", async () => {
      mockInvoke.mockRejectedValue(new Error("Device enumeration failed"));

      const { result } = renderHook(
        () => useAudioDevices({ autoRefresh: false }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual([]);
      expect(result.current.error?.message).toBe("Device enumeration failed");
    });

    it("refetch function triggers new fetch", async () => {
      const initialDevices = [
        { name: "Built-in Microphone", isDefault: true },
      ];
      const updatedDevices = [
        { name: "Built-in Microphone", isDefault: true },
        { name: "USB Microphone", isDefault: false },
      ];

      mockInvoke.mockResolvedValueOnce(initialDevices);
      mockInvoke.mockResolvedValueOnce(updatedDevices);

      const { result } = renderHook(
        () => useAudioDevices({ autoRefresh: false }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual(initialDevices);

      await act(async () => {
        result.current.refetch();
      });

      await waitFor(() => {
        expect(result.current.devices).toEqual(updatedDevices);
      });
    });

    it("returns empty array when no devices found", async () => {
      mockInvoke.mockResolvedValue([]);

      const { result } = renderHook(
        () => useAudioDevices({ autoRefresh: false }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });

  describe("periodic refresh with fake timers", () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("refreshes periodically when autoRefresh is enabled", async () => {
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      renderHook(
        () => useAudioDevices({ autoRefresh: true, refreshInterval: 1000 }),
        { wrapper: createWrapper() }
      );

      // Initial fetch
      await vi.runOnlyPendingTimersAsync();
      const initialCallCount = mockInvoke.mock.calls.length;

      // Advance timer by 1 second
      await vi.advanceTimersByTimeAsync(1000);
      expect(mockInvoke.mock.calls.length).toBeGreaterThan(initialCallCount);
    });

    it("does not refresh periodically when autoRefresh is disabled", async () => {
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      renderHook(() => useAudioDevices({ autoRefresh: false }), {
        wrapper: createWrapper(),
      });

      await vi.runOnlyPendingTimersAsync();
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance timer - should not trigger additional fetches
      await vi.advanceTimersByTimeAsync(10000);
      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });
  });
});
