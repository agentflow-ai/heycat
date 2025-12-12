import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { TranscriptionNotification } from "./TranscriptionNotification";
import * as useTranscriptionModule from "../hooks/useTranscription";

vi.mock("../hooks/useTranscription");

const mockUseTranscription = vi.mocked(useTranscriptionModule.useTranscription);

describe("TranscriptionNotification", () => {
  const defaultMock: useTranscriptionModule.UseTranscriptionResult = {
    isTranscribing: false,
    transcribedText: null,
    error: null,
    durationMs: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockUseTranscription.mockReturnValue(defaultMock);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("is hidden when no transcription or error", () => {
    const { container } = render(<TranscriptionNotification />);
    expect(container.firstChild).toBeNull();
  });

  it("displays success notification with transcribed text preview", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: "Hello world this is a test",
    });

    render(<TranscriptionNotification />);

    expect(
      screen.getByText("Hello world this is a test — Copied to clipboard")
    ).toBeDefined();
  });

  it("truncates long transcribed text", () => {
    const longText =
      "This is a very long transcription that exceeds fifty characters and should be truncated";
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: longText,
    });

    render(<TranscriptionNotification />);

    expect(screen.getByRole("status").textContent).toContain("...");
    expect(screen.getByRole("status").textContent).toContain(
      "Copied to clipboard"
    );
  });

  it("displays error notification with error message", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      error: "Model not available",
    });

    render(<TranscriptionNotification />);

    expect(screen.getByRole("alert").textContent).toContain(
      "Model not available"
    );
  });

  it("error notification has dismiss button", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      error: "Model not available",
    });

    render(<TranscriptionNotification />);

    const dismissButton = screen.getByRole("button", {
      name: "Dismiss error",
    });
    expect(dismissButton).toBeDefined();
  });

  it("dismisses error when dismiss button clicked", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      error: "Model not available",
    });

    render(<TranscriptionNotification />);

    const dismissButton = screen.getByRole("button", {
      name: "Dismiss error",
    });
    fireEvent.click(dismissButton);

    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("auto-dismisses success notification after delay", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: "Hello world",
    });

    render(<TranscriptionNotification autoDismissDelay={3000} />);

    expect(screen.getByRole("status")).toBeDefined();

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(screen.queryByRole("status")).toBeNull();
  });

  it("success notification has correct aria-live attribute", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: "Hello",
    });

    render(<TranscriptionNotification />);

    expect(screen.getByRole("status").getAttribute("aria-live")).toBe("polite");
  });

  it("error notification has correct aria-live attribute", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      error: "Error",
    });

    render(<TranscriptionNotification />);

    expect(screen.getByRole("alert").getAttribute("aria-live")).toBe(
      "assertive"
    );
  });

  it("applies custom className", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: "Hello",
    });

    const { container } = render(
      <TranscriptionNotification className="custom-class" />
    );

    expect((container.firstChild as HTMLElement)?.className).toContain("custom-class");
  });

  it("shows error instead of success when both present", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      transcribedText: "Hello",
      error: "Error occurred",
    });

    render(<TranscriptionNotification />);

    expect(screen.getByRole("alert")).toBeDefined();
    expect(screen.queryByText(/Copied to clipboard/)).toBeNull();
  });
});
