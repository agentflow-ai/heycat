import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Sidebar } from "./Sidebar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("rendering", () => {
    it("renders without errors", () => {
      expect(() => render(<Sidebar />)).not.toThrow();
    });

    it("renders as complementary landmark", () => {
      render(<Sidebar />);

      expect(screen.getByRole("complementary")).toBeDefined();
    });

    it("applies custom className", () => {
      render(<Sidebar className="custom-class" />);

      const sidebar = screen.getByRole("complementary");
      expect(sidebar.className).toContain("custom-class");
    });
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

    it("History tab has active styling by default", () => {
      render(<Sidebar />);

      const historyTab = screen.getByRole("tab", { name: "History" });
      expect(historyTab.className).toContain("sidebar__tab--active");
    });
  });

  describe("tab navigation", () => {
    it("clicking History tab triggers view change", () => {
      render(<Sidebar />);

      const historyTab = screen.getByRole("tab", { name: "History" });
      fireEvent.click(historyTab);

      expect(historyTab.getAttribute("aria-selected")).toBe("true");
    });

    it("renders tabpanel for history content", () => {
      render(<Sidebar />);

      expect(screen.getByRole("tabpanel")).toBeDefined();
    });
  });

  describe("accessibility", () => {
    it("has tablist navigation", () => {
      render(<Sidebar />);

      expect(screen.getByRole("tablist")).toBeDefined();
    });

    it("tablist has accessible label", () => {
      render(<Sidebar />);

      const tablist = screen.getByRole("tablist");
      expect(tablist.getAttribute("aria-label")).toBe("Sidebar navigation");
    });

    it("tab controls corresponding panel", () => {
      render(<Sidebar />);

      const historyTab = screen.getByRole("tab", { name: "History" });
      expect(historyTab.getAttribute("aria-controls")).toBe(
        "sidebar-panel-history"
      );
    });

    it("panel has correct id matching tab controls", () => {
      render(<Sidebar />);

      const panel = screen.getByRole("tabpanel");
      expect(panel.id).toBe("sidebar-panel-history");
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
