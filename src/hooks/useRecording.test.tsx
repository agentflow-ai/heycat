import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useRecording, useRecordingState, useStartRecording, useStopRecording } from "./useRecording";
import type { ReactNode } from "react";

// Mock Tauri APIs
const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Create a wrapper with QueryClientProvider
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

describe("useRecording", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("useRecordingState", () => {
    it("returns recording state from backend query", async () => {
      mockInvoke.mockResolvedValue({ state: "Recording" });

      const { result } = renderHook(() => useRecordingState(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isRecording).toBe(true);
      expect(result.current.isProcessing).toBe(false);
      expect(result.current.error).toBeNull();
      expect(mockInvoke).toHaveBeenCalledWith("get_recording_state");
    });

    it("reports processing state correctly", async () => {
      mockInvoke.mockResolvedValue({ state: "Processing" });

      const { result } = renderHook(() => useRecordingState(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isRecording).toBe(false);
      expect(result.current.isProcessing).toBe(true);
    });

    it("reports idle state when not recording or processing", async () => {
      mockInvoke.mockResolvedValue({ state: "Idle" });

      const { result } = renderHook(() => useRecordingState(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isRecording).toBe(false);
      expect(result.current.isProcessing).toBe(false);
    });
  });

  describe("useStartRecording", () => {
    it("calls start_recording with device name", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { result } = renderHook(() => useStartRecording(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync("MacBook Pro Mic");
      });

      expect(mockInvoke).toHaveBeenCalledWith("start_recording", {
        deviceName: "MacBook Pro Mic",
      });
    });

    it("captures error when start fails", async () => {
      mockInvoke.mockRejectedValue(new Error("Microphone not found"));

      const { result } = renderHook(() => useStartRecording(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync();
        } catch {
          // Expected to throw
        }
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      expect(result.current.error?.message).toBe("Microphone not found");
    });
  });

  describe("useStopRecording", () => {
    it("calls stop_recording", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { result } = renderHook(() => useStopRecording(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync();
      });

      expect(mockInvoke).toHaveBeenCalledWith("stop_recording");
    });
  });

  describe("useRecording (convenience hook)", () => {
    it("user can start and stop recording", async () => {
      mockInvoke.mockImplementation((command: string) => {
        if (command === "get_recording_state") {
          return Promise.resolve({ state: "Idle" });
        }
        return Promise.resolve(undefined);
      });

      const { result } = renderHook(() => useRecording(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isRecording).toBe(false);
      });

      // User starts recording
      await act(async () => {
        await result.current.startRecording();
      });

      expect(mockInvoke).toHaveBeenCalledWith("start_recording", {
        deviceName: undefined,
      });

      // User stops recording
      await act(async () => {
        await result.current.stopRecording();
      });

      expect(mockInvoke).toHaveBeenCalledWith("stop_recording");
    });

    it("passes device name to start_recording", async () => {
      mockInvoke.mockResolvedValue({ state: "Idle" });

      const { result } = renderHook(
        () => useRecording({ deviceName: "External Mic" }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isRecording).toBe(false);
      });

      await act(async () => {
        await result.current.startRecording();
      });

      expect(mockInvoke).toHaveBeenCalledWith("start_recording", {
        deviceName: "External Mic",
      });
    });

    it("user sees error when starting recording fails", async () => {
      mockInvoke.mockImplementation((command: string) => {
        if (command === "get_recording_state") {
          return Promise.resolve({ state: "Idle" });
        }
        if (command === "start_recording") {
          return Promise.reject(new Error("Microphone not found"));
        }
        return Promise.resolve(undefined);
      });

      const { result } = renderHook(() => useRecording(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isRecording).toBe(false);
      });

      await act(async () => {
        await result.current.startRecording();
      });

      expect(result.current.error).toBe("Microphone not found");
    });

    it("user sees error when stopping recording fails", async () => {
      mockInvoke.mockImplementation((command: string) => {
        if (command === "get_recording_state") {
          return Promise.resolve({ state: "Recording" });
        }
        if (command === "stop_recording") {
          return Promise.reject(new Error("Cannot stop - no active recording"));
        }
        return Promise.resolve(undefined);
      });

      const { result } = renderHook(() => useRecording(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isRecording).toBe(true);
      });

      await act(async () => {
        await result.current.stopRecording();
      });

      await waitFor(() => {
        expect(result.current.error).toBe("Cannot stop - no active recording");
      });
    });
  });
});
