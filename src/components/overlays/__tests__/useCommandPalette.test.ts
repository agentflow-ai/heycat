import { renderHook, act } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useCommandPalette } from "../useCommandPalette";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("useCommandPalette", () => {
  it("opens palette with Cmd+K shortcut", () => {
    const { result } = renderHook(() => useCommandPalette());

    expect(result.current.isOpen).toBe(false);

    // Simulate Cmd+K
    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        metaKey: true,
        bubbles: true,
      });
      document.dispatchEvent(event);
    });

    expect(result.current.isOpen).toBe(true);
  });

  it("toggles palette state with Cmd+K", () => {
    const { result } = renderHook(() => useCommandPalette());

    // Open
    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        metaKey: true,
        bubbles: true,
      });
      document.dispatchEvent(event);
    });
    expect(result.current.isOpen).toBe(true);

    // Close (toggle)
    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        metaKey: true,
        bubbles: true,
      });
      document.dispatchEvent(event);
    });
    expect(result.current.isOpen).toBe(false);
  });

  it("opens palette with Ctrl+K (Windows/Linux support)", () => {
    const { result } = renderHook(() => useCommandPalette());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        ctrlKey: true,
        bubbles: true,
      });
      document.dispatchEvent(event);
    });

    expect(result.current.isOpen).toBe(true);
  });

  it("calls onOpen callback when opening", () => {
    const onOpen = vi.fn();
    const { result } = renderHook(() => useCommandPalette({ onOpen }));

    act(() => {
      result.current.open();
    });

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("calls onClose callback when closing", () => {
    const onClose = vi.fn();
    const { result } = renderHook(() => useCommandPalette({ onClose }));

    // First open it
    act(() => {
      result.current.open();
    });

    // Then close
    act(() => {
      result.current.close();
    });

    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
