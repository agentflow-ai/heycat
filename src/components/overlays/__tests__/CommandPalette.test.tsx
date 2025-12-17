import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { CommandPalette } from "../CommandPalette";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("CommandPalette", () => {
  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    onCommandExecute: vi.fn(),
  };

  it("focuses search input when opened", async () => {
    render(<CommandPalette {...defaultProps} />);

    await waitFor(() => {
      expect(document.activeElement).toBe(
        screen.getByPlaceholderText("Search commands...")
      );
    });
  });

  it("filters commands as user types", async () => {
    const user = userEvent.setup();
    render(<CommandPalette {...defaultProps} />);

    // Initially shows all commands
    expect(screen.getByText("Start Recording")).toBeDefined();
    expect(screen.getByText("Go to Dashboard")).toBeDefined();

    // Type to filter
    await user.type(
      screen.getByPlaceholderText("Search commands..."),
      "record"
    );

    // Should show only matching commands
    expect(screen.getByText("Start Recording")).toBeDefined();
    expect(screen.getByText("Stop Recording")).toBeDefined();
    expect(screen.queryByText("Go to Dashboard")).toBeNull();
  });

  it("shows 'No results' when search has no matches", async () => {
    const user = userEvent.setup();
    render(<CommandPalette {...defaultProps} />);

    await user.type(
      screen.getByPlaceholderText("Search commands..."),
      "xyznonexistent"
    );

    expect(screen.getByText("No results found")).toBeDefined();
  });

  it("selects first command by default and updates on hover", async () => {
    const user = userEvent.setup();
    render(<CommandPalette {...defaultProps} />);

    // First item should be selected by default
    const firstOption = screen.getByText("Start Recording").closest("[role='option']");
    expect(firstOption?.getAttribute("aria-selected")).toBe("true");

    // Hover over another command to change selection
    const secondOption = screen.getByText("Stop Recording").closest("[role='option']");
    await user.hover(secondOption!);

    expect(secondOption?.getAttribute("aria-selected")).toBe("true");
    expect(firstOption?.getAttribute("aria-selected")).toBe("false");
  });

  it("executes command on Enter key", async () => {
    const user = userEvent.setup();
    const handleExecute = vi.fn();
    const handleClose = vi.fn();

    render(
      <CommandPalette
        isOpen={true}
        onClose={handleClose}
        onCommandExecute={handleExecute}
      />
    );

    // Wait for focus, then press Enter to execute first command
    const input = screen.getByPlaceholderText("Search commands...");
    await waitFor(() => expect(document.activeElement).toBe(input));
    await user.type(input, "{Enter}", { skipClick: true });

    expect(handleExecute).toHaveBeenCalledWith("start-recording");
    expect(handleClose).toHaveBeenCalled();
  });

  it("closes on Escape key", async () => {
    const user = userEvent.setup();
    const handleClose = vi.fn();

    render(
      <CommandPalette
        isOpen={true}
        onClose={handleClose}
        onCommandExecute={vi.fn()}
      />
    );

    // Wait for focus, then press Escape
    const input = screen.getByPlaceholderText("Search commands...");
    await waitFor(() => expect(document.activeElement).toBe(input));
    await user.type(input, "{Escape}", { skipClick: true });

    expect(handleClose).toHaveBeenCalled();
  });

  it("closes when clicking backdrop", async () => {
    const user = userEvent.setup();
    const handleClose = vi.fn();

    render(
      <CommandPalette
        isOpen={true}
        onClose={handleClose}
        onCommandExecute={vi.fn()}
      />
    );

    // Click the backdrop (the outer dialog element)
    const backdrop = screen.getByRole("dialog");
    await user.click(backdrop);

    expect(handleClose).toHaveBeenCalled();
  });

  it("executes command on click", async () => {
    const user = userEvent.setup();
    const handleExecute = vi.fn();
    const handleClose = vi.fn();

    render(
      <CommandPalette
        isOpen={true}
        onClose={handleClose}
        onCommandExecute={handleExecute}
      />
    );

    await user.click(screen.getByText("Go to Dashboard"));

    expect(handleExecute).toHaveBeenCalledWith("go-dashboard");
    expect(handleClose).toHaveBeenCalled();
  });

  it("groups commands by category with labels", () => {
    render(<CommandPalette {...defaultProps} />);

    // Check category labels are shown
    expect(screen.getByText("Actions")).toBeDefined();
    expect(screen.getByText("Navigation")).toBeDefined();
    expect(screen.getByText("Settings")).toBeDefined();
    expect(screen.getByText("Help")).toBeDefined();
  });

  it("displays keyboard shortcuts for commands that have them", () => {
    render(<CommandPalette {...defaultProps} />);

    // Start Recording has shortcut ⌘⇧R
    const startRecordingItem = screen.getByText("Start Recording").closest("[role='option']");
    expect(startRecordingItem?.textContent).toContain("⌘⇧R");

    // Go to Settings has shortcut ⌘,
    const settingsItem = screen.getByText("Go to Settings").closest("[role='option']");
    expect(settingsItem?.textContent).toContain("⌘,");
  });

  it("renders nothing when closed", () => {
    render(
      <CommandPalette
        isOpen={false}
        onClose={vi.fn()}
        onCommandExecute={vi.fn()}
      />
    );

    expect(screen.queryByRole("dialog")).toBeNull();
  });
});
