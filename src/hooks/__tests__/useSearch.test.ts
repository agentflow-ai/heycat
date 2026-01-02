/**
 * Tests for useSearch hook.
 */

import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSearch } from "../useSearch";

interface TestItem {
  id: string;
  name: string;
  description: string;
}

describe("useSearch", () => {
  const items: TestItem[] = [
    { id: "1", name: "Apple", description: "A red fruit" },
    { id: "2", name: "Banana", description: "A yellow fruit" },
    { id: "3", name: "Cherry", description: "A small red fruit" },
  ];

  const getSearchableText = (item: TestItem) =>
    `${item.name} ${item.description}`;

  it("returns all items when query is empty", () => {
    const { result } = renderHook(() =>
      useSearch({ items, getSearchableText })
    );

    expect(result.current.filteredItems).toHaveLength(3);
    expect(result.current.isSearching).toBe(false);
    expect(result.current.hasResults).toBe(true);
  });

  it("filters items by search query", () => {
    const { result } = renderHook(() =>
      useSearch({ items, getSearchableText })
    );

    act(() => {
      result.current.setQuery("red");
    });

    expect(result.current.filteredItems).toHaveLength(2);
    expect(result.current.filteredItems.map((i) => i.name)).toEqual([
      "Apple",
      "Cherry",
    ]);
    expect(result.current.isSearching).toBe(true);
  });

  it("performs case-insensitive search", () => {
    const { result } = renderHook(() =>
      useSearch({ items, getSearchableText })
    );

    act(() => {
      result.current.setQuery("BANANA");
    });

    expect(result.current.filteredItems).toHaveLength(1);
    expect(result.current.filteredItems[0].name).toBe("Banana");
  });

  it("clears search query", () => {
    const { result } = renderHook(() =>
      useSearch({ items, getSearchableText, initialQuery: "apple" })
    );

    expect(result.current.query).toBe("apple");

    act(() => {
      result.current.clearSearch();
    });

    expect(result.current.query).toBe("");
    expect(result.current.filteredItems).toHaveLength(3);
  });

  it("handles no results", () => {
    const { result } = renderHook(() =>
      useSearch({ items, getSearchableText })
    );

    act(() => {
      result.current.setQuery("xyz");
    });

    expect(result.current.filteredItems).toHaveLength(0);
    expect(result.current.hasResults).toBe(false);
  });
});
