import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { RecordingIndicator } from "./RecordingIndicator";
import * as useRecordingModule from "../hooks/useRecording";

vi.mock("../hooks/useRecording");

const mockUseRecording = vi.mocked(useRecordingModule.useRecording);

describe("RecordingIndicator", () => {
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

  it("renders idle state correctly when isRecording is false", () => {
    render(<RecordingIndicator />);

    expect(screen.getByText("Idle")).toBeDefined();
    expect(screen.getByRole("status").className).toContain(
      "recording-indicator--idle"
    );
  });

  it("renders recording state with red indicator when isRecording is true", () => {
    mockUseRecording.mockReturnValue({
      ...defaultMock,
      isRecording: true,
    });

    render(<RecordingIndicator />);

    expect(screen.getByText("Recording")).toBeDefined();
    expect(screen.getByRole("status").className).toContain(
      "recording-indicator--recording"
    );
  });

  it("shows error message when error prop is set", () => {
    mockUseRecording.mockReturnValue({
      ...defaultMock,
      error: "Microphone access denied",
    });

    render(<RecordingIndicator />);

    expect(screen.getByRole("alert").textContent).toBe(
      "Microphone access denied"
    );
  });

  it("has ARIA live region for accessibility", () => {
    render(<RecordingIndicator />);

    const status = screen.getByRole("status");
    expect(status.getAttribute("aria-live")).toBe("polite");
  });

  it("announces state via aria-label", () => {
    render(<RecordingIndicator />);

    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Recording status: Idle"
    );
  });

  it("updates aria-label when recording", () => {
    mockUseRecording.mockReturnValue({
      ...defaultMock,
      isRecording: true,
    });

    render(<RecordingIndicator />);

    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Recording status: Recording"
    );
  });

  it("applies custom className", () => {
    render(<RecordingIndicator className="custom-class" />);

    expect(screen.getByRole("status").className).toContain("custom-class");
  });

  it("hides dot from screen readers", () => {
    render(<RecordingIndicator />);

    const dot = document.querySelector(".recording-indicator__dot");
    expect(dot?.getAttribute("aria-hidden")).toBe("true");
  });

  it("shows blocked state when isBlocked is true", () => {
    render(<RecordingIndicator isBlocked={true} />);

    expect(screen.getByText("Recording blocked")).toBeDefined();
    expect(screen.getByRole("status").className).toContain(
      "recording-indicator--blocked"
    );
  });

  it("updates aria-label when blocked", () => {
    render(<RecordingIndicator isBlocked={true} />);

    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Recording status: Recording blocked"
    );
  });

  it("blocked state takes priority over recording state", () => {
    mockUseRecording.mockReturnValue({
      ...defaultMock,
      isRecording: true,
    });

    render(<RecordingIndicator isBlocked={true} />);

    expect(screen.getByText("Recording blocked")).toBeDefined();
    expect(screen.getByRole("status").className).toContain(
      "recording-indicator--blocked"
    );
  });
});
