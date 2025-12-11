/* v8 ignore file -- @preserve */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import App from "./App";
import * as useRecordingModule from "./hooks/useRecording";

vi.mock("./hooks/useRecording");
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

const mockUseRecording = vi.mocked(useRecordingModule.useRecording);

describe("App Integration", () => {
  const defaultMock: useRecordingModule.UseRecordingResult = {
    isRecording: false,
    error: null,
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    lastRecording: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseRecording.mockReturnValue(defaultMock);
  });

  it("renders RecordingIndicator component without errors", () => {
    render(<App />);

    const indicator = document.querySelector(".recording-indicator");
    expect(indicator).not.toBeNull();
    expect(screen.getByText("Idle")).toBeDefined();
  });

  it("RecordingIndicator is visible with app-recording-indicator class", () => {
    render(<App />);

    const indicator = document.querySelector(".recording-indicator");
    expect(indicator?.className).toContain("app-recording-indicator");
  });

  it("syncs state when backend emits recording events", async () => {
    const { rerender } = render(<App />);

    expect(screen.getByText("Idle")).toBeDefined();

    mockUseRecording.mockReturnValue({
      ...defaultMock,
      isRecording: true,
    });

    rerender(<App />);

    expect(screen.getByText("Recording")).toBeDefined();
    const indicator = document.querySelector(".recording-indicator");
    expect(indicator?.className).toContain("recording-indicator--recording");
  });

  it("handles multiple rapid state changes correctly", async () => {
    const { rerender } = render(<App />);

    const states = [true, false, true, false, true, false];
    for (const isRecording of states) {
      mockUseRecording.mockReturnValue({
        ...defaultMock,
        isRecording,
      });
      rerender(<App />);
    }

    expect(screen.getByText("Idle")).toBeDefined();
  });

  it("RecordingIndicator remains visible after other UI interactions", async () => {
    render(<App />);

    const input = screen.getByPlaceholderText("Enter a name...");
    await act(async () => {
      input.focus();
    });

    const indicator = document.querySelector(".recording-indicator");
    expect(indicator).not.toBeNull();
    expect(screen.getByText("Idle")).toBeDefined();
  });

  it("App renders without console errors", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(<App />);

    expect(consoleSpy).not.toHaveBeenCalled();
    consoleSpy.mockRestore();
  });
});
