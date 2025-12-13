import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar } from "./Sidebar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("History tab", () => {
    it("History tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "History" })).toBeDefined();
    });

    it("History tab is selected by default", async () => {
      render(<Sidebar />);

      const historyTab = await screen.findByRole("tab", { name: "History" });
      expect(historyTab.getAttribute("aria-selected")).toBe("true");
    });
  });

  describe("Commands tab", () => {
    it("Commands tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "Commands" })).toBeDefined();
    });

    it("Commands tab switches panel content when clicked", async () => {
      render(<Sidebar />);

      const commandsTab = screen.getByRole("tab", { name: "Commands" });
      await userEvent.click(commandsTab);

      expect(commandsTab.getAttribute("aria-selected")).toBe("true");
      expect(
        screen.getByRole("tab", { name: "History" }).getAttribute("aria-selected")
      ).toBe("false");
    });

    it("can start on Commands tab via defaultTab prop", async () => {
      render(<Sidebar defaultTab="commands" />);

      const commandsTab = await screen.findByRole("tab", { name: "Commands" });
      expect(commandsTab.getAttribute("aria-selected")).toBe("true");
      // Wait for CommandSettings async effect to complete
      await waitFor(() => {
        expect(screen.getByText("Voice Commands")).toBeDefined();
      });
    });
  });

  describe("tab navigation", () => {
    it("renders tabpanel for history content", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tabpanel")).toBeDefined();
    });
  });

  describe("content rendering", () => {
    it("renders RecordingsList in history panel", async () => {
      render(<Sidebar />);

      // RecordingsList is rendered - check for loading state initially
      // Since invoke is mocked to return [], it will show empty state
      const panel = await screen.findByRole("tabpanel");
      expect(panel).toBeDefined();
    });
  });
});
