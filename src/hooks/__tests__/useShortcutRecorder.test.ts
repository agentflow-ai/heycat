/**
 * Tests for useShortcutRecorder hook.
 *
 * Note: This hook relies heavily on Tauri APIs which are mocked.
 * Tests focus on state management and callback behavior.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useShortcutRecorder } from "../useShortcutRecorder";

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

describe("useShortcutRecorder", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(mockUnlisten);
  });

  it("initializes with default state", () => {
    const { result } = renderHook(() => useShortcutRecorder());

    expect(result.current.isRecording).toBe(false);
    expect(result.current.recordedShortcut).toBeNull();
    expect(result.current.permissionError).toBeNull();
  });

  it("starts recording and calls backend", async () => {
    const { result } = renderHook(() => useShortcutRecorder());

    await act(async () => {
      await result.current.startRecording();
    });

    expect(result.current.isRecording).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("start_shortcut_recording");
  });

  it("stops recording and calls backend", async () => {
    const { result } = renderHook(() => useShortcutRecorder());

    await act(async () => {
      await result.current.startRecording();
    });

    await act(async () => {
      await result.current.stopRecording();
    });

    expect(result.current.isRecording).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith("stop_shortcut_recording");
  });

  it("handles permission error", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Accessibility permission required"));

    const { result } = renderHook(() => useShortcutRecorder());

    await act(async () => {
      await result.current.startRecording();
    });

    expect(result.current.isRecording).toBe(false);
    expect(result.current.permissionError).toContain("Accessibility permission");
  });

  it("clears recorded shortcut", async () => {
    const { result } = renderHook(() => useShortcutRecorder());

    // Simulate having a recorded shortcut by mocking the event
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_eventName, callback) => {
      eventCallback = callback;
      return Promise.resolve(mockUnlisten);
    });

    await act(async () => {
      await result.current.startRecording();
    });

    // Simulate key event
    if (eventCallback) {
      act(() => {
        eventCallback!({
          payload: {
            key_code: 0,
            key_name: "A",
            command: true,
            command_left: true,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: false,
            shift_left: false,
            shift_right: false,
            fn_key: false,
            pressed: true,
            is_media_key: false,
          },
        });
      });
    }

    await waitFor(() => {
      expect(result.current.recordedShortcut).not.toBeNull();
    });

    act(() => {
      result.current.clearRecordedShortcut();
    });

    expect(result.current.recordedShortcut).toBeNull();
  });

  it("opens accessibility preferences", async () => {
    const { result } = renderHook(() => useShortcutRecorder());

    await act(async () => {
      await result.current.openAccessibilityPreferences();
    });

    expect(mockInvoke).toHaveBeenCalledWith("open_accessibility_preferences");
  });
});
