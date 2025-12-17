import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AudioLevelMeter, MiniAudioMeter } from "./AudioLevelMeter";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("AudioLevelMeter", () => {
  it("displays current level with accessible meter role", () => {
    render(<AudioLevelMeter level={50} />);
    const meter = screen.getByRole("meter");
    expect(meter.getAttribute("aria-valuenow")).toBe("50");
    expect(meter.getAttribute("aria-valuemin")).toBe("0");
    expect(meter.getAttribute("aria-valuemax")).toBe("100");
  });

  it("clamps level to valid range (0-100)", () => {
    const { rerender } = render(<AudioLevelMeter level={150} />);
    expect(screen.getByRole("meter").getAttribute("aria-valuenow")).toBe("100");

    rerender(<AudioLevelMeter level={-20} />);
    expect(screen.getByRole("meter").getAttribute("aria-valuenow")).toBe("0");
  });

  it("updates visual width when level changes", () => {
    const { rerender, container } = render(<AudioLevelMeter level={25} />);
    const fill = container.querySelector('[style*="width"]');
    expect(fill?.getAttribute("style")).toContain("width: 25%");

    rerender(<AudioLevelMeter level={75} />);
    expect(fill?.getAttribute("style")).toContain("width: 75%");
  });
});

describe("MiniAudioMeter", () => {
  it("renders with meter role for accessibility", () => {
    render(<MiniAudioMeter level={60} />);
    const meter = screen.getByRole("meter");
    expect(meter.getAttribute("aria-valuenow")).toBe("60");
  });

  it("clamps level to valid range", () => {
    render(<MiniAudioMeter level={200} />);
    expect(screen.getByRole("meter").getAttribute("aria-valuenow")).toBe("100");
  });
});
