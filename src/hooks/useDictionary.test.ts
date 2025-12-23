import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useDictionary } from "./useDictionary";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      children
    );
  };
}

describe("useDictionary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("entries query", () => {
    it("exposes loading state during fetch", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));
      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      expect(result.current.entries.isLoading).toBe(true);
    });

    it("fetches entries from backend", async () => {
      const mockEntries = [
        { id: "1", trigger: "brb", expansion: "be right back" },
        { id: "2", trigger: "omw", expansion: "on my way" },
      ];
      mockInvoke.mockResolvedValue(mockEntries);

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith("list_dictionary_entries");
      expect(result.current.entries.data).toEqual(mockEntries);
    });

    it("returns entries with suffix and autoEnter fields", async () => {
      const mockEntries = [
        {
          id: "1",
          trigger: "brb",
          expansion: "be right back",
          suffix: ".",
          autoEnter: true,
        },
        {
          id: "2",
          trigger: "omw",
          expansion: "on my way",
          suffix: null,
          autoEnter: false,
        },
      ];
      mockInvoke.mockResolvedValue(mockEntries);

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      expect(result.current.entries.data).toEqual(mockEntries);
      expect(result.current.entries.data?.[0].suffix).toBe(".");
      expect(result.current.entries.data?.[0].autoEnter).toBe(true);
      expect(result.current.entries.data?.[1].suffix).toBeNull();
      expect(result.current.entries.data?.[1].autoEnter).toBe(false);
    });
  });

  describe("addEntry mutation", () => {
    it("calls backend with trigger and expansion", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      const newEntry = { id: "1", trigger: "brb", expansion: "be right back" };
      mockInvoke.mockResolvedValueOnce(newEntry); // Add mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.addEntry.mutateAsync({
          trigger: "brb",
          expansion: "be right back",
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
        trigger: "brb",
        expansion: "be right back",
      });
    });

    it("passes suffix to backend when provided", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      const newEntry = {
        id: "1",
        trigger: "brb",
        expansion: "be right back",
        suffix: ".",
        autoEnter: false,
      };
      mockInvoke.mockResolvedValueOnce(newEntry); // Add mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.addEntry.mutateAsync({
          trigger: "brb",
          expansion: "be right back",
          suffix: ".",
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
        trigger: "brb",
        expansion: "be right back",
        suffix: ".",
      });
    });

    it("passes autoEnter to backend when provided", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      const newEntry = {
        id: "1",
        trigger: "brb",
        expansion: "be right back",
        autoEnter: true,
      };
      mockInvoke.mockResolvedValueOnce(newEntry); // Add mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.addEntry.mutateAsync({
          trigger: "brb",
          expansion: "be right back",
          autoEnter: true,
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
        trigger: "brb",
        expansion: "be right back",
        autoEnter: true,
      });
    });

    it("exposes error state when mutation fails", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      mockInvoke.mockRejectedValueOnce(new Error("Trigger cannot be empty")); // Failed mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      // Use mutate instead of mutateAsync to allow error state to be captured
      act(() => {
        result.current.addEntry.mutate({
          trigger: "",
          expansion: "test",
        });
      });

      await waitFor(() => {
        expect(result.current.addEntry.isError).toBe(true);
      });

      expect(result.current.addEntry.error?.message).toBe(
        "Trigger cannot be empty"
      );
    });
  });

  describe("updateEntry mutation", () => {
    it("calls backend with id, trigger, and expansion", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      mockInvoke.mockResolvedValueOnce(undefined); // Update mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.updateEntry.mutateAsync({
          id: "1",
          trigger: "brb",
          expansion: "be right back soon",
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
        id: "1",
        trigger: "brb",
        expansion: "be right back soon",
      });
    });

    it("passes suffix to backend when updating", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      mockInvoke.mockResolvedValueOnce(undefined); // Update mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.updateEntry.mutateAsync({
          id: "1",
          trigger: "brb",
          expansion: "be right back",
          suffix: "!",
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
        id: "1",
        trigger: "brb",
        expansion: "be right back",
        suffix: "!",
      });
    });

    it("passes autoEnter to backend when updating", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      mockInvoke.mockResolvedValueOnce(undefined); // Update mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.updateEntry.mutateAsync({
          id: "1",
          trigger: "brb",
          expansion: "be right back",
          autoEnter: true,
        });
      });

      expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
        id: "1",
        trigger: "brb",
        expansion: "be right back",
        autoEnter: true,
      });
    });
  });

  describe("deleteEntry mutation", () => {
    it("calls backend with id", async () => {
      mockInvoke.mockResolvedValueOnce([]); // Initial query
      mockInvoke.mockResolvedValueOnce(undefined); // Delete mutation

      const { result } = renderHook(() => useDictionary(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.entries.isLoading).toBe(false);
      });

      await act(async () => {
        await result.current.deleteEntry.mutateAsync({ id: "1" });
      });

      expect(mockInvoke).toHaveBeenCalledWith("delete_dictionary_entry", {
        id: "1",
      });
    });
  });
});
