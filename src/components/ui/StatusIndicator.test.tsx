import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusIndicator, RecordingDot } from "./StatusIndicator";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("StatusIndicator", () => {
  it("displays recording status with accessible label", () => {
    render(<StatusIndicator variant="recording" />);
    const status = screen.getByRole("status");
    expect(status.getAttribute("aria-label")).toBe("Status: Recording");
    expect(screen.getByText("Recording")).toBeDefined();
  });

  it("allows custom label override", () => {
    render(<StatusIndicator variant="recording" label="Active" />);
    expect(screen.getByRole("status").getAttribute("aria-label")).toBe("Status: Active");
    expect(screen.getByText("Active")).toBeDefined();
  });
});

describe("RecordingDot", () => {
  it("renders visible dot element", () => {
    render(<RecordingDot data-testid="dot" />);
    expect(screen.getByTestId("dot")).toBeDefined();
  });

  it("is hidden from screen readers", () => {
    render(<RecordingDot data-testid="dot" />);
    expect(screen.getByTestId("dot").getAttribute("aria-hidden")).toBe("true");
  });
});
