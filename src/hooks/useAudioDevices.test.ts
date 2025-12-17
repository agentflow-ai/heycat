import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useAudioDevices } from "./useAudioDevices";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

describe("useAudioDevices", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns loading state initially", () => {
    mockInvoke.mockReturnValue(new Promise(() => {}));
    const { result } = renderHook(() => useAudioDevices());

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

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(mockInvoke).toHaveBeenCalledWith("list_audio_devices");
    expect(result.current.devices).toEqual(mockDevices);
    expect(result.current.error).toBeNull();
  });

  it("handles fetch error", async () => {
    mockInvoke.mockRejectedValue(new Error("Device enumeration failed"));

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.devices).toEqual([]);
    expect(result.current.error?.message).toBe("Device enumeration failed");
  });

  it("handles non-Error error", async () => {
    mockInvoke.mockRejectedValue("String error");

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error?.message).toBe("String error");
  });

  it("refresh function refetches devices", async () => {
    const initialDevices = [{ name: "Built-in Microphone", isDefault: true }];
    const updatedDevices = [
      { name: "Built-in Microphone", isDefault: true },
      { name: "USB Microphone", isDefault: false },
    ];

    mockInvoke.mockResolvedValueOnce(initialDevices);
    mockInvoke.mockResolvedValueOnce(updatedDevices);

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.devices).toEqual(initialDevices);

    await act(async () => {
      result.current.refresh();
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.devices).toEqual(updatedDevices);
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("clears error on successful refresh", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("First error"));
    mockInvoke.mockResolvedValueOnce([
      { name: "Built-in Microphone", isDefault: true },
    ]);

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).not.toBeNull();

    await act(async () => {
      result.current.refresh();
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
  });

  it("returns empty array when no devices found", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useAudioDevices());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.devices).toEqual([]);
    expect(result.current.error).toBeNull();
  });
});
