/**
 * Tests for useEditableItem hook.
 */

import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useEditableItem } from "../useEditableItem";

interface TestItem {
  id: string;
  name: string;
  value: number;
}

interface TestEditValues {
  name: string;
  value: number;
  // Index signature for useEditableItem compatibility
  [key: string]: string | number;
}

describe("useEditableItem", () => {
  const getInitialValues = (item: TestItem): TestEditValues => ({
    name: item.name,
    value: item.value,
  });

  const testItem: TestItem = { id: "1", name: "Test", value: 42 };

  it("starts with no item being edited", () => {
    const { result } = renderHook(() =>
      useEditableItem<TestItem, TestEditValues>({ getInitialValues })
    );

    expect(result.current.editingId).toBeNull();
    expect(result.current.isEditing("1")).toBe(false);
  });

  it("starts editing an item", () => {
    const { result } = renderHook(() =>
      useEditableItem<TestItem, TestEditValues>({ getInitialValues })
    );

    act(() => {
      result.current.startEdit("1", testItem);
    });

    expect(result.current.editingId).toBe("1");
    expect(result.current.isEditing("1")).toBe(true);
    expect(result.current.isEditing("2")).toBe(false);
    expect(result.current.editValues).toEqual({ name: "Test", value: 42 });
  });

  it("cancels editing", () => {
    const { result } = renderHook(() =>
      useEditableItem<TestItem, TestEditValues>({ getInitialValues })
    );

    act(() => {
      result.current.startEdit("1", testItem);
    });

    expect(result.current.editingId).toBe("1");

    act(() => {
      result.current.cancelEdit();
    });

    expect(result.current.editingId).toBeNull();
    expect(result.current.isEditing("1")).toBe(false);
  });

  it("updates edit values", () => {
    const { result } = renderHook(() =>
      useEditableItem<TestItem, TestEditValues>({ getInitialValues })
    );

    act(() => {
      result.current.startEdit("1", testItem);
    });

    act(() => {
      result.current.setEditValue("name", "Updated");
    });

    expect(result.current.editValues.name).toBe("Updated");
    expect(result.current.editValues.value).toBe(42);
  });

  it("switches to editing a different item", () => {
    const item2: TestItem = { id: "2", name: "Other", value: 100 };

    const { result } = renderHook(() =>
      useEditableItem<TestItem, TestEditValues>({ getInitialValues })
    );

    act(() => {
      result.current.startEdit("1", testItem);
    });

    expect(result.current.isEditing("1")).toBe(true);

    act(() => {
      result.current.startEdit("2", item2);
    });

    expect(result.current.isEditing("1")).toBe(false);
    expect(result.current.isEditing("2")).toBe(true);
    expect(result.current.editValues).toEqual({ name: "Other", value: 100 });
  });
});
