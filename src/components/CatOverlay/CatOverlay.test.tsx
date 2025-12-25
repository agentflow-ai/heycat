import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, act } from "@testing-library/react";
import { CatOverlay } from "./CatOverlay";

// Mock Tauri event API
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

// Track callbacks for overlay-mode event
let overlayModeCallback: ((event: { payload: { mode: string; isMicUnavailable: boolean } }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (eventName: string, callback: (event: { payload: unknown }) => void) => {
    mockListen(eventName, callback);
    if (eventName === "overlay-mode") {
      overlayModeCallback = callback as (event: { payload: { mode: string; isMicUnavailable: boolean } }) => void;
    }
    return Promise.resolve(mockUnlisten);
  },
}));

// Helper function to trigger overlay mode changes within act
const setOverlayMode = (mode: string, isMicUnavailable: boolean) => {
  act(() => {
    overlayModeCallback!({ payload: { mode, isMicUnavailable } });
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

  // Event listener setup test removed per TESTING.md:
  // - Testing listener setup is testing React framework internals, not user behavior

  it("updates to listening mode class on overlay-mode event", async () => {
    const { container } = render(<CatOverlay />);

    await waitFor(() => {
      expect(overlayModeCallback).not.toBeNull();
    });

    setOverlayMode("listening", false);

    const overlay = container.querySelector(".cat-overlay");
    expect(overlay?.className).toContain("cat-overlay--listening");
  });

  it("applies unavailable class when mic is unavailable", async () => {
    const { container } = render(<CatOverlay />);

    await waitFor(() => {
      expect(overlayModeCallback).not.toBeNull();
    });

    setOverlayMode("listening", true);

    const overlay = container.querySelector(".cat-overlay");
    expect(overlay?.className).toContain("cat-overlay--unavailable");
  });

  // Event listener cleanup test removed per TESTING.md:
  // - Testing cleanup is testing React framework internals, not user behavior

  it("transitions from listening to recording mode", async () => {
    const { container } = render(<CatOverlay />);

    await waitFor(() => {
      expect(overlayModeCallback).not.toBeNull();
    });

    // Start in listening mode
    setOverlayMode("listening", false);

    const overlay = container.querySelector(".cat-overlay");
    expect(overlay?.className).toContain("cat-overlay--listening");

    // Transition to recording mode
    setOverlayMode("recording", false);

    expect(overlay?.className).toContain("cat-overlay--recording");
    expect(overlay?.className).not.toContain("cat-overlay--listening");
  });
});
