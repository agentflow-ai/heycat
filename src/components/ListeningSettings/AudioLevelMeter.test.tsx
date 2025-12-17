import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { AudioLevelMeter } from "./AudioLevelMeter";

describe("AudioLevelMeter", () => {
  it("renders level bar at 0%", () => {
    render(<AudioLevelMeter level={0} isMonitoring={false} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar).toBeDefined();
    expect(progressbar.getAttribute("aria-valuenow")).toBe("0");
    expect(progressbar.style.width).toBe("0%");
  });

  it("renders level bar at 50%", () => {
    render(<AudioLevelMeter level={50} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.getAttribute("aria-valuenow")).toBe("50");
    expect(progressbar.style.width).toBe("50%");
  });

  it("renders level bar at 100%", () => {
    render(<AudioLevelMeter level={100} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.getAttribute("aria-valuenow")).toBe("100");
    expect(progressbar.style.width).toBe("100%");
  });

  it("shows safe zone styling for low levels", () => {
    render(<AudioLevelMeter level={30} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--safe");
  });

  it("shows optimal zone styling for medium levels", () => {
    render(<AudioLevelMeter level={60} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--optimal");
  });

  it("shows clipping zone styling for high levels", () => {
    render(<AudioLevelMeter level={90} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--clipping");
  });

  it("shows Monitoring status when active", () => {
    render(<AudioLevelMeter level={50} isMonitoring={true} />);

    expect(screen.getByText("Monitoring")).toBeDefined();
  });

  it("shows Idle status when not monitoring", () => {
    render(<AudioLevelMeter level={0} isMonitoring={false} />);

    expect(screen.getByText("Idle")).toBeDefined();
  });

  it("has proper accessibility attributes", () => {
    render(<AudioLevelMeter level={75} isMonitoring={true} />);

    const progressbar = screen.getByRole("progressbar");
    expect(progressbar.getAttribute("aria-valuemin")).toBe("0");
    expect(progressbar.getAttribute("aria-valuemax")).toBe("100");
    expect(progressbar.getAttribute("aria-label")).toBe("Audio input level");
  });

  it("has zone markers for optimal and clipping thresholds", () => {
    const { container } = render(
      <AudioLevelMeter level={50} isMonitoring={true} />
    );

    const markers = container.querySelectorAll(".audio-level-meter__marker");
    expect(markers.length).toBe(2);
  });

  it("applies correct class at boundary values", () => {
    // Test at exact boundary - 50 should be safe
    const { rerender } = render(
      <AudioLevelMeter level={50} isMonitoring={true} />
    );
    let progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--safe");

    // Test at 51 - should be optimal
    rerender(<AudioLevelMeter level={51} isMonitoring={true} />);
    progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--optimal");

    // Test at 85 - should still be optimal
    rerender(<AudioLevelMeter level={85} isMonitoring={true} />);
    progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--optimal");

    // Test at 86 - should be clipping
    rerender(<AudioLevelMeter level={86} isMonitoring={true} />);
    progressbar = screen.getByRole("progressbar");
    expect(progressbar.className).toContain("audio-level-meter__fill--clipping");
  });
});
