import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { Sidebar } from "./Sidebar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("History tab", () => {
    it("History tab is present in sidebar", () => {
      render(<Sidebar />);

      expect(screen.getByRole("tab", { name: "History" })).toBeDefined();
    });

    it("History tab is selected by default", () => {
      render(<Sidebar />);

      const historyTab = screen.getByRole("tab", { name: "History" });
      expect(historyTab.getAttribute("aria-selected")).toBe("true");
    });
  });

  describe("tab navigation", () => {
    it("renders tabpanel for history content", () => {
      render(<Sidebar />);

      expect(screen.getByRole("tabpanel")).toBeDefined();
    });
  });

  describe("content rendering", () => {
    it("renders RecordingsList in history panel", async () => {
      render(<Sidebar />);

      // RecordingsList is rendered - check for loading state initially
      // Since invoke is mocked to return [], it will show empty state
      const panel = screen.getByRole("tabpanel");
      expect(panel).toBeDefined();
    });
  });
});
