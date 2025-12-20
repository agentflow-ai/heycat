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
vi.mock("./useListening", () => ({
  useListening: vi.fn(),
}));

import { useRecording } from "./useRecording";
import { useTranscription } from "./useTranscription";
import { useListening } from "./useListening";

const mockUseRecording = vi.mocked(useRecording);
const mockUseTranscription = vi.mocked(useTranscription);
const mockUseListening = vi.mocked(useListening);

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
    mockUseListening.mockReturnValue({
      isListening: false,
      isWakeWordDetected: false,
      isMicAvailable: true,
      error: null,
      enableListening: vi.fn(),
      disableListening: vi.fn(),
    });
  });

  it("returns idle when nothing is active", () => {
    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("idle");
  });

  it("returns listening when listening is active", () => {
    mockUseListening.mockReturnValue({
      isListening: true,
      isWakeWordDetected: false,
      isMicAvailable: true,
      error: null,
      enableListening: vi.fn(),
      disableListening: vi.fn(),
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.status).toBe("listening");
  });

  it("returns recording when recording (priority over listening)", () => {
    mockUseListening.mockReturnValue({
      isListening: true,
      isWakeWordDetected: false,
      isMicAvailable: true,
      error: null,
      enableListening: vi.fn(),
      disableListening: vi.fn(),
    });
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

  it("returns processing when transcribing (priority over listening)", () => {
    mockUseListening.mockReturnValue({
      isListening: true,
      isWakeWordDetected: false,
      isMicAvailable: true,
      error: null,
      enableListening: vi.fn(),
      disableListening: vi.fn(),
    });
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
    mockUseListening.mockReturnValue({
      isListening: true,
      isWakeWordDetected: false,
      isMicAvailable: true,
      error: null,
      enableListening: vi.fn(),
      disableListening: vi.fn(),
    });

    const { result } = renderHook(() => useAppStatus());
    expect(result.current.isRecording).toBe(true);
    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.isListening).toBe(true);
  });
});
