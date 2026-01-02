import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, act } from "@testing-library/react";
import { CatOverlay } from "./CatOverlay";

// Mock Tauri event API
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

// Track callbacks for overlay_mode event
let overlayModeCallback: ((event: { payload: { mode: string } }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (eventName: string, callback: (event: { payload: unknown }) => void) => {
    mockListen(eventName, callback);
    if (eventName === "overlay_mode") {
      overlayModeCallback = callback as (event: { payload: { mode: string } }) => void;
    }
    return Promise.resolve(mockUnlisten);
  },
}));

// Helper function to trigger overlay mode changes within act
const setOverlayMode = (mode: string) => {
  act(() => {
    overlayModeCallback!({ payload: { mode } });
  });
};

describe("CatOverlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    overlayModeCallback = null;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders video element", () => {
    const { container } = render(<CatOverlay />);

    const video = container.querySelector("video");
    expect(video).toBeDefined();
    expect(video).not.toBeNull();
    expect(video?.loop).toBe(true);
    expect(video?.muted).toBe(true);
  });

  it("applies recording mode class by default", () => {
    const { container } = render(<CatOverlay />);

    const overlay = container.querySelector(".cat-overlay");
    expect(overlay?.className).toContain("cat-overlay--recording");
  });

  it("transitions between different recording modes", async () => {
    const { container } = render(<CatOverlay />);

    await waitFor(() => {
      expect(overlayModeCallback).not.toBeNull();
    });

    // Recording mode is default
    const overlay = container.querySelector(".cat-overlay");
    expect(overlay?.className).toContain("cat-overlay--recording");

    // Transition to hidden and back to recording
    setOverlayMode("hidden");
    expect(overlay?.className).toContain("cat-overlay--hidden");

    setOverlayMode("recording");
    expect(overlay?.className).toContain("cat-overlay--recording");
    expect(overlay?.className).not.toContain("cat-overlay--hidden");
  });
});
