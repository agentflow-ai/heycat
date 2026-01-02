/**
 * Tests for useWindowContextForm hook.
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useWindowContextForm } from "../useWindowContextForm";

describe("useWindowContextForm", () => {
  it("initializes with default values", () => {
    const { result } = renderHook(() => useWindowContextForm());

    expect(result.current.values.name).toBe("");
    expect(result.current.values.appName).toBe("");
    expect(result.current.values.commandMode).toBe("merge");
    expect(result.current.nameError).toBeNull();
  });

  it("validates required name on submit", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useWindowContextForm({ onSubmit })
    );

    act(() => {
      result.current.setValue("appName", "Test App");
    });

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(false);
    expect(result.current.nameError).toBe("Name is required");
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("validates required appName on submit", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useWindowContextForm({ onSubmit })
    );

    act(() => {
      result.current.setValue("name", "My Context");
    });

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(false);
    expect(result.current.appNameError).toBe("App name is required");
  });

  it("validates regex pattern", async () => {
    const { result } = renderHook(() => useWindowContextForm());

    act(() => {
      result.current.setValue("name", "Test");
      result.current.setValue("appName", "App");
      result.current.setValue("titlePattern", "[invalid");
    });

    await act(async () => {
      await result.current.handleSubmit();
    });

    expect(result.current.patternError).toContain("Invalid regex");
  });

  it("calls onSubmit with valid values", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useWindowContextForm({ onSubmit })
    );

    act(() => {
      result.current.setValue("name", "Slack Context");
      result.current.setValue("appName", "Slack");
      result.current.setValue("titlePattern", ".*#general.*");
    });

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(true);
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "Slack Context",
        appName: "Slack",
        titlePattern: ".*#general.*",
      })
    );
  });

  it("handleAppSelect sets both appName and bundleId", () => {
    const { result } = renderHook(() => useWindowContextForm());

    act(() => {
      result.current.handleAppSelect("Slack", "com.tinyspeck.slackmacgap");
    });

    expect(result.current.values.appName).toBe("Slack");
    expect(result.current.values.bundleId).toBe("com.tinyspeck.slackmacgap");
  });

  it("clears bundleId when appName is manually changed", () => {
    const { result } = renderHook(() =>
      useWindowContextForm({
        initialValues: {
          appName: "Slack",
          bundleId: "com.tinyspeck.slackmacgap",
        },
      })
    );

    expect(result.current.values.bundleId).toBe("com.tinyspeck.slackmacgap");

    act(() => {
      result.current.setValue("appName", "Discord");
    });

    expect(result.current.values.appName).toBe("Discord");
    expect(result.current.values.bundleId).toBeUndefined();
  });
});
