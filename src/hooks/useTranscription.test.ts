import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTranscription } from "./useTranscription";
import { useAppStore } from "../stores/appStore";

describe("useTranscription", () => {
  beforeEach(() => {
    // Reset the store to initial state before each test
    useAppStore.setState({
      transcription: {
        isTranscribing: false,
        transcribedText: null,
        error: null,
        durationMs: null,
      },
    });
  });

  it("returns initial transcription state", () => {
    const { result } = renderHook(() => useTranscription());

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });

  it("reflects transcriptionStarted state from store", () => {
    const { result } = renderHook(() => useTranscription());

    act(() => {
      useAppStore.getState().transcriptionStarted();
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });

  it("reflects transcriptionCompleted state from store", () => {
    const { result } = renderHook(() => useTranscription());

    act(() => {
      useAppStore.getState().transcriptionStarted();
    });

    expect(result.current.isTranscribing).toBe(true);

    act(() => {
      useAppStore.getState().transcriptionCompleted("Hello, world!", 1234);
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBe("Hello, world!");
    expect(result.current.durationMs).toBe(1234);
    expect(result.current.error).toBeNull();
  });

  it("reflects transcriptionError state from store", () => {
    const { result } = renderHook(() => useTranscription());

    act(() => {
      useAppStore.getState().transcriptionStarted();
    });

    expect(result.current.isTranscribing).toBe(true);

    act(() => {
      useAppStore.getState().transcriptionError("Model not loaded");
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.error).toBe("Model not loaded");
  });

  it("clears previous result when new transcription starts", () => {
    const { result } = renderHook(() => useTranscription());

    // Complete first transcription
    act(() => {
      useAppStore.getState().transcriptionStarted();
      useAppStore.getState().transcriptionCompleted("First transcription", 1000);
    });

    expect(result.current.transcribedText).toBe("First transcription");
    expect(result.current.durationMs).toBe(1000);

    // Start new transcription - should clear previous result
    act(() => {
      useAppStore.getState().transcriptionStarted();
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });

  it("clears error when new transcription starts", () => {
    const { result } = renderHook(() => useTranscription());

    // First transcription errors
    act(() => {
      useAppStore.getState().transcriptionStarted();
      useAppStore.getState().transcriptionError("Transcription timed out");
    });

    expect(result.current.error).toBe("Transcription timed out");

    // Start new transcription - should clear error
    act(() => {
      useAppStore.getState().transcriptionStarted();
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.error).toBeNull();
  });
});
