/**
 * Tests for useDictionaryForm hook.
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useDictionaryForm } from "../useDictionaryForm";

describe("useDictionaryForm", () => {
  const existingTriggers = ["brb", "omw"];

  it("initializes with default values", () => {
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers })
    );

    expect(result.current.values.trigger).toBe("");
    expect(result.current.values.expansion).toBe("");
    expect(result.current.triggerError).toBeNull();
    expect(result.current.suffixError).toBeNull();
  });

  it("validates required trigger on submit", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers, onSubmit })
    );

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(false);
    expect(result.current.triggerError).toBe("Trigger is required");
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("validates duplicate trigger on submit", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers, onSubmit })
    );

    act(() => {
      result.current.setValue("trigger", "brb");
      result.current.setValue("expansion", "be right back");
    });

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(false);
    expect(result.current.triggerError).toBe("This trigger already exists");
  });

  it("validates suffix length", async () => {
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers })
    );

    act(() => {
      result.current.setValue("trigger", "test");
      result.current.setValue("suffix", "123456"); // Too long
    });

    await act(async () => {
      await result.current.handleSubmit();
    });

    expect(result.current.suffixError).toContain("5 characters or less");
  });

  it("calls onSubmit with valid values", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers, onSubmit })
    );

    act(() => {
      result.current.setValue("trigger", "lol");
      result.current.setValue("expansion", "laughing out loud");
    });

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(true);
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        trigger: "lol",
        expansion: "laughing out loud",
      })
    );
  });

  it("toggles settings panel", () => {
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers })
    );

    expect(result.current.isSettingsOpen).toBe(false);

    act(() => {
      result.current.toggleSettings();
    });

    expect(result.current.isSettingsOpen).toBe(true);

    act(() => {
      result.current.toggleSettings();
    });

    expect(result.current.isSettingsOpen).toBe(false);
  });

  it("computes hasSettings correctly", () => {
    const { result } = renderHook(() =>
      useDictionaryForm({ existingTriggers })
    );

    expect(result.current.hasSettings).toBe(false);

    act(() => {
      result.current.setValue("suffix", ".");
    });

    expect(result.current.hasSettings).toBe(true);
  });
});
