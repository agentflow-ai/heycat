/**
 * Tests for useDeleteConfirmation hook.
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useDeleteConfirmation } from "../useDeleteConfirmation";

describe("useDeleteConfirmation", () => {
  it("starts with no confirmation pending", () => {
    const { result } = renderHook(() => useDeleteConfirmation());

    expect(result.current.confirmingId).toBeNull();
    expect(result.current.isPending).toBe(false);
    expect(result.current.isConfirming("1")).toBe(false);
  });

  it("requests deletion of an item", () => {
    const { result } = renderHook(() => useDeleteConfirmation());

    act(() => {
      result.current.requestDelete("1");
    });

    expect(result.current.confirmingId).toBe("1");
    expect(result.current.isPending).toBe(true);
    expect(result.current.isConfirming("1")).toBe(true);
    expect(result.current.isConfirming("2")).toBe(false);
  });

  it("cancels pending deletion", () => {
    const { result } = renderHook(() => useDeleteConfirmation());

    act(() => {
      result.current.requestDelete("1");
    });

    expect(result.current.isPending).toBe(true);

    act(() => {
      result.current.cancelDelete();
    });

    expect(result.current.confirmingId).toBeNull();
    expect(result.current.isPending).toBe(false);
  });

  it("calls onConfirm when deletion is confirmed", async () => {
    const onConfirm = vi.fn();
    const { result } = renderHook(() =>
      useDeleteConfirmation({ onConfirm })
    );

    act(() => {
      result.current.requestDelete("1");
    });

    await act(async () => {
      await result.current.confirmDelete();
    });

    expect(onConfirm).toHaveBeenCalledWith("1");
    expect(result.current.confirmingId).toBeNull();
    expect(result.current.isPending).toBe(false);
  });

  it("does nothing if confirmDelete called without pending item", async () => {
    const onConfirm = vi.fn();
    const { result } = renderHook(() =>
      useDeleteConfirmation({ onConfirm })
    );

    await act(async () => {
      await result.current.confirmDelete();
    });

    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("replaces pending item when new deletion is requested", () => {
    const { result } = renderHook(() => useDeleteConfirmation());

    act(() => {
      result.current.requestDelete("1");
    });

    expect(result.current.isConfirming("1")).toBe(true);

    act(() => {
      result.current.requestDelete("2");
    });

    expect(result.current.isConfirming("1")).toBe(false);
    expect(result.current.isConfirming("2")).toBe(true);
    expect(result.current.confirmingId).toBe("2");
  });

  it("clears confirming state even when onConfirm rejects", async () => {
    const error = new Error("Delete failed");
    const onConfirm = vi.fn().mockRejectedValue(error);
    const { result } = renderHook(() =>
      useDeleteConfirmation({ onConfirm })
    );

    act(() => {
      result.current.requestDelete("1");
    });

    expect(result.current.isPending).toBe(true);

    // confirmDelete should not throw, but should still clear state
    await act(async () => {
      await expect(result.current.confirmDelete()).rejects.toThrow(error);
    });

    // State should be cleared even after error
    expect(result.current.confirmingId).toBeNull();
    expect(result.current.isPending).toBe(false);
  });
});
