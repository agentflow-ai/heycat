import { render, screen, act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { StatusPill, AutoTimerStatusPill } from "./StatusPill";

describe("StatusPill", () => {
  describe("renders correct state visualizations", () => {
    it("displays idle state with 'Ready' text and gray styling", () => {
      render(<StatusPill status="idle" />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-label")).toBe("Status: Ready");
      expect(screen.getByText("Ready")).toBeDefined();
      expect(pill.className).toContain("bg-neutral-400");
    });

    it("displays recording state with 'Recording' text, red styling, and pulse animation", () => {
      render(<StatusPill status="recording" />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-label")).toBe("Status: Recording");
      expect(screen.getByText("Recording")).toBeDefined();
      expect(pill.className).toContain("bg-recording");
      expect(pill.className).toContain("status-pill-pulse");
    });

    it("displays processing state with 'Processing...' text, amber styling, and spinner", () => {
      render(<StatusPill status="processing" />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-label")).toBe("Status: Processing...");
      expect(screen.getByText("Processing...")).toBeDefined();
      expect(pill.className).toContain("bg-processing");
      // Spinner should be present (Loader2 icon)
      const spinner = pill.querySelector("svg");
      expect(spinner).not.toBeNull();
      // SVG className is an SVGAnimatedString, use getAttribute instead
      expect(spinner?.getAttribute("class")).toContain("animate-spin");
    });
  });

  describe("recording duration timer", () => {
    it("shows duration timer when recording with duration provided", () => {
      render(<StatusPill status="recording" recordingDuration={65} />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-label")).toContain("duration 1:05");
      expect(screen.getByText("1:05")).toBeDefined();
    });

    it("formats duration correctly for various values", () => {
      const { rerender } = render(<StatusPill status="recording" recordingDuration={0} />);
      expect(screen.getByText("0:00")).toBeDefined();

      rerender(<StatusPill status="recording" recordingDuration={59} />);
      expect(screen.getByText("0:59")).toBeDefined();

      rerender(<StatusPill status="recording" recordingDuration={120} />);
      expect(screen.getByText("2:00")).toBeDefined();
    });

    it("does not show duration for non-recording states", () => {
      render(<StatusPill status="idle" recordingDuration={10} />);
      expect(screen.queryByText("0:10")).toBeNull();
    });
  });

  describe("custom label override", () => {
    it("displays custom label when provided", () => {
      render(<StatusPill status="idle" label="Standby" />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-label")).toBe("Status: Standby");
      expect(screen.getByText("Standby")).toBeDefined();
    });
  });

  describe("accessibility", () => {
    it("has role=status and aria-live for screen reader announcements", () => {
      render(<StatusPill status="recording" />);

      const pill = screen.getByRole("status");
      expect(pill.getAttribute("aria-live")).toBe("polite");
    });
  });
});

describe("AutoTimerStatusPill", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("auto-increments timer when in recording state", () => {
    render(<AutoTimerStatusPill status="recording" />);

    // Initially 0:00
    expect(screen.getByText("0:00")).toBeDefined();

    // After 1 second
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(screen.getByText("0:01")).toBeDefined();

    // After another 5 seconds
    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(screen.getByText("0:06")).toBeDefined();
  });

  it("resets timer when status changes away from recording", () => {
    const { rerender } = render(<AutoTimerStatusPill status="recording" />);

    // Advance timer
    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(screen.getByText("0:05")).toBeDefined();

    // Change to processing
    rerender(<AutoTimerStatusPill status="processing" />);
    // Duration should not be shown for processing
    expect(screen.queryByText("0:05")).toBeNull();

    // Back to recording starts from 0
    rerender(<AutoTimerStatusPill status="recording" />);
    expect(screen.getByText("0:00")).toBeDefined();
  });

  it("supports initial duration offset", () => {
    render(<AutoTimerStatusPill status="recording" initialDuration={60} />);

    expect(screen.getByText("1:00")).toBeDefined();

    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(screen.getByText("1:05")).toBeDefined();
  });
});
