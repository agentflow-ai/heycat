import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ShortcutEditor } from "./ShortcutEditor";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("ShortcutEditor", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    shortcutName: "Toggle Recording",
    currentShortcut: "⌘⇧R",
    onSave: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  describe("Theming", () => {
    it("renders hotkey display with theme-aware styling", () => {
      render(<ShortcutEditor {...defaultProps} />);

      const kbd = screen.getByText("⌘⇧R");
      // Verify it uses theme-aware classes instead of hardcoded colors
      expect(kbd.className).toContain("bg-surface-elevated");
      expect(kbd.className).toContain("text-text-primary");
      // Ensure it does NOT have the broken hardcoded style
      expect(kbd.className).not.toContain("bg-neutral-100");
    });
  });

  describe("Recording Mode - Global Shortcut Management", () => {
    it("suspends global shortcut when entering recording mode", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("suspend_recording_shortcut");
      });
    });

    it("shows recording state after clicking Record button", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(screen.getByText("Press your shortcut...")).toBeDefined();
        expect(screen.getByRole("button", { name: "Recording..." })).toBeDefined();
      });
    });

    it("resumes global shortcut when modal closes while suspended", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      const { rerender } = render(
        <ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />
      );

      // Enter recording mode (suspends shortcut)
      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("suspend_recording_shortcut");
      });

      // Clear mock to track resume call
      mockInvoke.mockClear();

      // Close modal
      rerender(
        <ShortcutEditor {...defaultProps} open={false} onOpenChange={onOpenChange} />
      );

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("resume_recording_shortcut");
      });
    });

    it("does not call suspend if not entering recording mode", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      // Click cancel without entering recording mode
      await user.click(screen.getByRole("button", { name: "Cancel" }));

      // suspend should not have been called
      expect(mockInvoke).not.toHaveBeenCalledWith("suspend_recording_shortcut");
    });
  });

  describe("Modal Behavior", () => {
    it("shows correct shortcut name in modal header", () => {
      render(<ShortcutEditor {...defaultProps} shortcutName="Custom Action" />);

      expect(screen.getByText(/Set a new shortcut for "Custom Action"/)).toBeDefined();
    });

    it("displays current shortcut initially", () => {
      render(<ShortcutEditor {...defaultProps} currentShortcut="⌘K" />);

      expect(screen.getByText("⌘K")).toBeDefined();
    });

    it("does not render when open is false", () => {
      render(<ShortcutEditor {...defaultProps} open={false} />);

      expect(screen.queryByText("Change Keyboard Shortcut")).toBeNull();
    });

    it("calls onOpenChange when Cancel is clicked", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(<ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />);

      await user.click(screen.getByRole("button", { name: "Cancel" }));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("calls onOpenChange when close button is clicked", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(<ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />);

      await user.click(screen.getByRole("button", { name: "Close" }));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("disables Save button when no changes have been made", () => {
      render(<ShortcutEditor {...defaultProps} />);

      const saveButton = screen.getByRole("button", { name: "Save" });
      expect(saveButton.hasAttribute("disabled")).toBe(true);
    });
  });
});
