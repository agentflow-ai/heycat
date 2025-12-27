import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useWindowContext } from "./useWindowContext";
import type { WindowContext } from "../types/windowContext";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

function createTestContext(overrides: Partial<WindowContext> = {}): WindowContext {
  return {
    id: "test-id",
    name: "Test Context",
    matcher: {
      appName: "TestApp",
      titlePattern: undefined,
      bundleId: undefined,
    },
    commandMode: "merge",
    dictionaryMode: "merge",
    commandIds: [],
    dictionaryEntryIds: [],
    enabled: true,
    priority: 0,
    ...overrides,
  };
}

describe("useWindowContext", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
        },
      },
    });
    vi.clearAllMocks();
  });

  afterEach(() => {
    queryClient.clear();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  describe("contexts query", () => {
    it("fetches and returns contexts list", async () => {
      const mockContexts = [createTestContext({ name: "Context 1" }), createTestContext({ id: "test-2", name: "Context 2" })];
      mockInvoke.mockResolvedValueOnce(mockContexts);

      const { result } = renderHook(() => useWindowContext(), { wrapper });

      await waitFor(() => {
        expect(result.current.contexts.isSuccess).toBe(true);
      });

      expect(result.current.contexts.data).toEqual(mockContexts);
      expect(mockInvoke).toHaveBeenCalledWith("list_window_contexts");
    });

    it("handles error when fetch fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Failed to fetch"));

      const { result } = renderHook(() => useWindowContext(), { wrapper });

      await waitFor(() => {
        expect(result.current.contexts.isError).toBe(true);
      });

      expect(result.current.contexts.error).toBeInstanceOf(Error);
    });
  });

  describe("addContext mutation", () => {
    it("calls add_window_context with correct parameters", async () => {
      const newContext = createTestContext();
      mockInvoke.mockResolvedValueOnce([]);
      mockInvoke.mockResolvedValueOnce(newContext);

      const { result } = renderHook(() => useWindowContext(), { wrapper });

      await waitFor(() => {
        expect(result.current.contexts.isSuccess).toBe(true);
      });

      await result.current.addContext.mutateAsync({
        name: "New Context",
        appName: "NewApp",
        titlePattern: ".*pattern.*",
        commandMode: "replace",
        dictionaryMode: "merge",
        priority: 5,
        enabled: true,
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_window_context", {
        name: "New Context",
        appName: "NewApp",
        titlePattern: ".*pattern.*",
        bundleId: undefined,
        commandMode: "replace",
        dictionaryMode: "merge",
        commandIds: undefined,
        dictionaryEntryIds: undefined,
        priority: 5,
        enabled: true,
      });
    });
  });

  describe("updateContext mutation", () => {
    it("calls update_window_context with correct parameters", async () => {
      mockInvoke.mockResolvedValueOnce([createTestContext()]);
      mockInvoke.mockResolvedValueOnce(undefined);

      const { result } = renderHook(() => useWindowContext(), { wrapper });

      await waitFor(() => {
        expect(result.current.contexts.isSuccess).toBe(true);
      });

      await result.current.updateContext.mutateAsync({
        id: "test-id",
        name: "Updated Context",
        appName: "UpdatedApp",
        commandMode: "replace",
        dictionaryMode: "replace",
        priority: 10,
        enabled: false,
      });

      expect(mockInvoke).toHaveBeenCalledWith("update_window_context", {
        id: "test-id",
        name: "Updated Context",
        appName: "UpdatedApp",
        titlePattern: undefined,
        bundleId: undefined,
        commandMode: "replace",
        dictionaryMode: "replace",
        commandIds: undefined,
        dictionaryEntryIds: undefined,
        priority: 10,
        enabled: false,
      });
    });
  });

  describe("deleteContext mutation", () => {
    it("calls delete_window_context with correct id", async () => {
      mockInvoke.mockResolvedValueOnce([createTestContext()]);
      mockInvoke.mockResolvedValueOnce(undefined);

      const { result } = renderHook(() => useWindowContext(), { wrapper });

      await waitFor(() => {
        expect(result.current.contexts.isSuccess).toBe(true);
      });

      await result.current.deleteContext.mutateAsync({ id: "test-id" });

      expect(mockInvoke).toHaveBeenCalledWith("delete_window_context", { id: "test-id" });
    });
  });
});
