import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useActiveWindow } from "./useActiveWindow";
import type { ActiveWindowChangedPayload } from "../types/windowContext";

// Mock @tauri-apps/api/event
const mockUnlisten = vi.fn();
let mockListeners: Map<string, (event: { payload: unknown }) => void> = new Map();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, callback: (event: { payload: unknown }) => void) => {
    mockListeners.set(eventName, callback);
    return Promise.resolve(mockUnlisten);
  }),
}));

describe("useActiveWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListeners.clear();
  });

  afterEach(() => {
    mockListeners.clear();
  });

  it("updates state when active_window_changed event is received", async () => {
    const { result } = renderHook(() => useActiveWindow());

    // Wait for listener to be registered
    await vi.waitFor(() => {
      expect(mockListeners.has("active_window_changed")).toBe(true);
    });

    const payload: ActiveWindowChangedPayload = {
      appName: "Visual Studio Code",
      bundleId: "com.microsoft.VSCode",
      windowTitle: "main.rs - heycat",
      matchedContextId: "ctx-123",
      matchedContextName: "VS Code",
    };

    act(() => {
      const listener = mockListeners.get("active_window_changed");
      listener?.({ payload });
    });

    expect(result.current.activeWindow).toEqual(payload);
    expect(result.current.matchedContextId).toBe("ctx-123");
    expect(result.current.matchedContextName).toBe("VS Code");
  });

  it("returns null for matched context when not present in payload", async () => {
    const { result } = renderHook(() => useActiveWindow());

    await vi.waitFor(() => {
      expect(mockListeners.has("active_window_changed")).toBe(true);
    });

    const payload: ActiveWindowChangedPayload = {
      appName: "Slack",
      windowTitle: "#general",
      // No matchedContextId or matchedContextName
    };

    act(() => {
      const listener = mockListeners.get("active_window_changed");
      listener?.({ payload });
    });

    expect(result.current.activeWindow).toEqual(payload);
    expect(result.current.matchedContextId).toBeNull();
    expect(result.current.matchedContextName).toBeNull();
  });
});
