import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { useAppStatus } from "./useAppStatus";

// Mock the individual hooks
vi.mock("./useRecording", () => ({
  useRecording: vi.fn(),
}));
vi.mock("./useTranscription", () => ({
  useTranscription: vi.fn(),
}));

import { useRecording } from "./useRecording";
import { useTranscription } from "./useTranscription";

const mockUseRecording = vi.mocked(useRecording);
const mockUseTranscription = vi.mocked(useTranscription);

describe("useAppStatus", () => {
  beforeEach(() => {
    // Default: all idle
    mockUseRecording.mockReturnValue({
      isRecording: false,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });
    mockUseTranscription.mockReturnValue({
      isTranscribing: false,
      transcribedText: null,
      error: null,
      durationMs: null,
    });
  });

  it("returns idle when nothing is active", () => {
    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("idle");
  });

  it("returns recording when recording", () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("recording");
  });

  it("returns processing when transcribing", () => {
    mockUseTranscription.mockReturnValue({
      isTranscribing: true,
      transcribedText: null,
      error: null,
      durationMs: null,
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("processing");
  });

  it("returns recording over processing (highest priority)", () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });
    mockUseTranscription.mockReturnValue({
      isTranscribing: true,
      transcribedText: null,
      error: null,
      durationMs: null,
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("recording");
  });

  it("exposes first available error from hooks", () => {
    mockUseRecording.mockReturnValue({
      isRecording: false,
      isProcessing: false,
      error: "Recording error",
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.error).toBe("Recording error");
  });

  it("exposes individual state booleans", () => {
    mockUseRecording.mockReturnValue({
      isRecording: true,
      isProcessing: false,
      error: null,
      startRecording: vi.fn(),
      stopRecording: vi.fn(),
      isStarting: false,
      isStopping: false,
    });
    mockUseTranscription.mockReturnValue({
      isTranscribing: true,
      transcribedText: null,
      error: null,
      durationMs: null,
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.isRecording).toBe(true);
    expect(result.current.isTranscribing).toBe(true);
  });
});
