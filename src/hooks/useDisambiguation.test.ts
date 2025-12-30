import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useDisambiguation } from "./useDisambiguation";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();
const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("useDisambiguation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(mockUnlisten);
    mockInvoke.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("updates state when command_ambiguous event fires", async () => {
    let ambiguousCallback: ((event: {
      payload: {
        transcription: string;
        candidates: Array<{ id: string; trigger: string; confidence: number }>;
      };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "command_ambiguous") {
          ambiguousCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useDisambiguation());

    await waitFor(() => {
      expect(ambiguousCallback).not.toBeNull();
    });

    act(() => {
      ambiguousCallback!({
        payload: {
          transcription: "open app",
          candidates: [
            { id: "1", trigger: "open slack", confidence: 0.85 },
            { id: "2", trigger: "open safari", confidence: 0.82 },
          ],
        },
      });
    });

    expect(result.current.isAmbiguous).toBe(true);
    expect(result.current.transcription).toBe("open app");
    expect(result.current.candidates).toHaveLength(2);
    expect(result.current.candidates[0].trigger).toBe("open slack");
    expect(result.current.candidates[1].trigger).toBe("open safari");
  });

  it("clears state when command_executed event fires", async () => {
    let ambiguousCallback: ((event: { payload: unknown }) => void) | null = null;
    let executedCallback: (() => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event?: { payload: unknown }) => void
      ) => {
        if (eventName === "command_ambiguous") {
          ambiguousCallback = callback;
        } else if (eventName === "command_executed") {
          executedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useDisambiguation());

    await waitFor(() => {
      expect(ambiguousCallback).not.toBeNull();
      expect(executedCallback).not.toBeNull();
    });

    // First trigger ambiguous state
    act(() => {
      ambiguousCallback!({
        payload: {
          transcription: "open app",
          candidates: [{ id: "1", trigger: "open slack", confidence: 0.85 }],
        },
      });
    });

    expect(result.current.isAmbiguous).toBe(true);

    // Then command executes successfully
    act(() => {
      executedCallback!();
    });

    expect(result.current.isAmbiguous).toBe(false);
    expect(result.current.transcription).toBeNull();
    expect(result.current.candidates).toEqual([]);
  });

  it("clears state when command_failed event fires", async () => {
    let ambiguousCallback: ((event: { payload: unknown }) => void) | null = null;
    let failedCallback: (() => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event?: { payload: unknown }) => void
      ) => {
        if (eventName === "command_ambiguous") {
          ambiguousCallback = callback;
        } else if (eventName === "command_failed") {
          failedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useDisambiguation());

    await waitFor(() => {
      expect(ambiguousCallback).not.toBeNull();
      expect(failedCallback).not.toBeNull();
    });

    // First trigger ambiguous state
    act(() => {
      ambiguousCallback!({
        payload: {
          transcription: "open app",
          candidates: [{ id: "1", trigger: "open slack", confidence: 0.85 }],
        },
      });
    });

    expect(result.current.isAmbiguous).toBe(true);

    // Then command fails
    act(() => {
      failedCallback!();
    });

    expect(result.current.isAmbiguous).toBe(false);
    expect(result.current.transcription).toBeNull();
    expect(result.current.candidates).toEqual([]);
  });

  it("dismiss clears state", async () => {
    let ambiguousCallback: ((event: { payload: unknown }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "command_ambiguous") {
          ambiguousCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useDisambiguation());

    await waitFor(() => {
      expect(ambiguousCallback).not.toBeNull();
    });

    // First trigger ambiguous state
    act(() => {
      ambiguousCallback!({
        payload: {
          transcription: "open app",
          candidates: [{ id: "1", trigger: "open slack", confidence: 0.85 }],
        },
      });
    });

    expect(result.current.isAmbiguous).toBe(true);

    // Dismiss
    act(() => {
      result.current.dismiss();
    });

    expect(result.current.isAmbiguous).toBe(false);
    expect(result.current.transcription).toBeNull();
    expect(result.current.candidates).toEqual([]);
  });
});
