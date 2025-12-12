import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { TranscriptionIndicator } from "./TranscriptionIndicator";
import * as useTranscriptionModule from "../hooks/useTranscription";

vi.mock("../hooks/useTranscription");

const mockUseTranscription = vi.mocked(useTranscriptionModule.useTranscription);

describe("TranscriptionIndicator", () => {
  const defaultMock: useTranscriptionModule.UseTranscriptionResult = {
    isTranscribing: false,
    transcribedText: null,
    error: null,
    durationMs: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseTranscription.mockReturnValue(defaultMock);
  });

  it("is hidden when isTranscribing is false", () => {
    const { container } = render(<TranscriptionIndicator />);
    expect(container.firstChild).toBeNull();
  });

  it("shows loading state when isTranscribing is true", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator />);

    expect(screen.getByText("Transcribing...")).toBeDefined();
    expect(screen.getByRole("status")).toBeDefined();
  });

  it("has aria-busy attribute when transcribing", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator />);

    expect(screen.getByRole("status").getAttribute("aria-busy")).toBe("true");
  });

  it("has aria-live=polite for accessibility", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator />);

    expect(screen.getByRole("status").getAttribute("aria-live")).toBe("polite");
  });

  it("has aria-label describing transcription status", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator />);

    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Transcribing audio"
    );
  });

  it("applies custom className", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator className="custom-class" />);

    expect(screen.getByRole("status").className).toContain("custom-class");
  });

  it("hides spinner from screen readers", () => {
    mockUseTranscription.mockReturnValue({
      ...defaultMock,
      isTranscribing: true,
    });

    render(<TranscriptionIndicator />);

    const spinner = document.querySelector(
      ".transcription-indicator__spinner"
    );
    expect(spinner?.getAttribute("aria-hidden")).toBe("true");
  });
});
