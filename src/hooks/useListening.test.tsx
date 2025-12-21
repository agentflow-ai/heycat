import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useListening, useListeningStatus, useEnableListening, useDisableListening } from "./useListening";
import { useAppStore } from "../stores/appStore";
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

describe("useListening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset Zustand store
    useAppStore.setState({
      listening: { isWakeWordDetected: false },
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("useListeningStatus", () => {
    it("returns listening status from backend query", async () => {
      mockInvoke.mockResolvedValue({
        enabled: true,
        active: true,
        micAvailable: true,
      });

      const { result } = renderHook(() => useListeningStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isListening).toBe(true);
      expect(result.current.isMicAvailable).toBe(true);
      expect(result.current.error).toBeNull();
      expect(mockInvoke).toHaveBeenCalledWith("get_listening_status");
    });

    it("reports mic unavailable when backend returns micAvailable: false", async () => {
      mockInvoke.mockResolvedValue({
        enabled: false,
        active: false,
        micAvailable: false,
      });

      const { result } = renderHook(() => useListeningStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isListening).toBe(false);
      expect(result.current.isMicAvailable).toBe(false);
    });
  });

  describe("useEnableListening", () => {
    it("calls enable_listening with device name", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { result } = renderHook(() => useEnableListening(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync("MacBook Pro Mic");
      });

      expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
        deviceName: "MacBook Pro Mic",
      });
    });

    it("captures error when enable fails", async () => {
      mockInvoke.mockRejectedValue(new Error("Cannot enable while recording"));

      const { result } = renderHook(() => useEnableListening(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(undefined);
        } catch {
          // Expected to throw
        }
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      expect(result.current.error?.message).toBe("Cannot enable while recording");
    });
  });

  describe("useDisableListening", () => {
    it("calls disable_listening", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { result } = renderHook(() => useDisableListening(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(undefined);
      });

      expect(mockInvoke).toHaveBeenCalledWith("disable_listening");
    });
  });

  describe("useListening (convenience hook)", () => {
    it("user can enable and disable listening mode", async () => {
      mockInvoke.mockImplementation((command: string) => {
        if (command === "get_listening_status") {
          return Promise.resolve({ enabled: false, active: false, micAvailable: true });
        }
        return Promise.resolve(undefined);
      });

      const { result } = renderHook(() => useListening(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isMicAvailable).toBe(true);
      });

      // User enables listening
      await act(async () => {
        await result.current.enableListening();
      });

      expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
        deviceName: undefined,
      });

      // User disables listening
      await act(async () => {
        await result.current.disableListening();
      });

      expect(mockInvoke).toHaveBeenCalledWith("disable_listening");
    });

    it("passes device name to enable_listening", async () => {
      mockInvoke.mockResolvedValue({ enabled: false, active: false, micAvailable: true });

      const { result } = renderHook(
        () => useListening({ deviceName: "External Mic" }),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isMicAvailable).toBe(true);
      });

      await act(async () => {
        await result.current.enableListening();
      });

      expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
        deviceName: "External Mic",
      });
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

      const { result } = renderHook(() => useListening(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isMicAvailable).toBe(true);
      });

      await act(async () => {
        await result.current.enableListening();
      });

      expect(result.current.error).toBe("Cannot enable while recording");
    });

    it("reads wake word detection state from Zustand", async () => {
      mockInvoke.mockResolvedValue({ enabled: false, active: false, micAvailable: true });

      const { result } = renderHook(() => useListening(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isMicAvailable).toBe(true);
      });

      expect(result.current.isWakeWordDetected).toBe(false);

      // Simulate Event Bridge updating Zustand
      act(() => {
        useAppStore.getState().wakeWordDetected();
      });

      expect(result.current.isWakeWordDetected).toBe(true);

      // Simulate Event Bridge clearing after timeout
      act(() => {
        useAppStore.getState().clearWakeWord();
      });

      expect(result.current.isWakeWordDetected).toBe(false);
    });
  });
});
