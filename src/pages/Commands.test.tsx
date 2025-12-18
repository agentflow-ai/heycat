import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { Commands, type CommandDto } from "./Commands";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock("../components/overlays", () => ({
  useToast: () => ({
    toast: mockToast,
    dismiss: vi.fn(),
    dismissAll: vi.fn(),
  }),
}));

// Sample command data
const sampleCommands: CommandDto[] = [
  {
    id: "1",
    trigger: "open slack",
    action_type: "open_app",
    parameters: { app: "Slack" },
    enabled: true,
  },
  {
    id: "2",
    trigger: "type my email",
    action_type: "type_text",
    parameters: { text: "hello@example.com" },
    enabled: true,
  },
  {
    id: "3",
    trigger: "volume up",
    action_type: "system_control",
    parameters: { control: "volume_up" },
    enabled: false,
  },
];

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("renders page with header, search, and new command button", async () => {
    render(<Commands />);

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Voice Commands" })
      ).toBeDefined();
    });

    expect(
      screen.getByText("Create custom voice commands to control your Mac.")
    ).toBeDefined();
    // There may be multiple New Command buttons (header + empty state)
    const newCommandButtons = screen.getAllByRole("button", { name: /new command/i });
    expect(newCommandButtons.length).toBeGreaterThan(0);
    expect(
      screen.getByRole("textbox", { name: /search commands/i })
    ).toBeDefined();
  });

  it("shows empty state when no commands exist", async () => {
    mockInvoke.mockResolvedValue([]);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("No voice commands yet")).toBeDefined();
    });

    expect(
      screen.getByText("Create your first command to get started")
    ).toBeDefined();
    // Empty state also has a New Command button
    const newCommandButtons = screen.getAllByRole("button", {
      name: /new command/i,
    });
    expect(newCommandButtons.length).toBeGreaterThan(0);
  });

  it("displays commands list with toggle, trigger, and action type", async () => {
    mockInvoke.mockResolvedValue(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    // Check all commands are displayed
    expect(screen.getByText('"type my email"')).toBeDefined();
    expect(screen.getByText('"volume up"')).toBeDefined();

    // Check action type badges
    expect(screen.getByText("Open App")).toBeDefined();
    expect(screen.getByText("Type Text")).toBeDefined();
    expect(screen.getByText("System Control")).toBeDefined();

    // Check toggle switches exist
    const toggles = screen.getAllByRole("switch");
    expect(toggles).toHaveLength(3);
  });

  it("filters commands by search query", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    // Search for "email"
    const searchInput = screen.getByRole("textbox", {
      name: /search commands/i,
    });
    await user.type(searchInput, "email");

    // Only matching command should be visible
    expect(screen.queryByText('"open slack"')).toBeNull();
    expect(screen.getByText('"type my email"')).toBeDefined();
    expect(screen.queryByText('"volume up"')).toBeNull();
  });

  it("shows no results message when search has no matches", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    const searchInput = screen.getByRole("textbox", {
      name: /search commands/i,
    });
    await user.type(searchInput, "nonexistent");

    expect(screen.getByText('No commands match "nonexistent"')).toBeDefined();
  });

  it("toggles command enabled state", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(sampleCommands) // Initial load
      .mockResolvedValueOnce({ ...sampleCommands[0], enabled: false }); // Toggle response

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    // Find the toggle for "open slack" and click it
    const slackToggle = screen.getByRole("switch", {
      name: /disable open slack/i,
    });
    await user.click(slackToggle);

    expect(mockInvoke).toHaveBeenCalledWith("update_command", {
      input: {
        id: "1",
        trigger: "open slack",
        action_type: "open_app",
        parameters: { app: "Slack" },
        enabled: false,
      },
    });
  });

  it("opens modal for new command when clicking New Command button", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue([]);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("No voice commands yet")).toBeDefined();
    });

    // Click first "New Command" button (header button)
    const newCommandButtons = screen.getAllByRole("button", { name: /new command/i });
    await user.click(newCommandButtons[0]);

    // Modal should open
    expect(
      screen.getByRole("dialog", { name: /create voice command/i })
    ).toBeDefined();
    expect(
      screen.getByRole("textbox", { name: /trigger phrase/i })
    ).toBeDefined();
  });

  it("opens modal for editing when clicking edit button", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    await user.click(screen.getByRole("button", { name: /edit open slack/i }));

    // Modal should open with command data
    const dialog = screen.getByRole("dialog", { name: /edit voice command/i });
    expect(dialog).toBeDefined();

    const triggerInput = within(dialog).getByRole("textbox", {
      name: /trigger phrase/i,
    });
    expect(triggerInput).toHaveValue("open slack");
  });

  it("creates new command through modal form", async () => {
    const user = userEvent.setup();
    const newCommand: CommandDto = {
      id: "new-1",
      trigger: "open spotify",
      action_type: "open_app",
      parameters: { app: "Spotify" },
      enabled: true,
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial load
      .mockResolvedValueOnce(newCommand); // add_command response

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("No voice commands yet")).toBeDefined();
    });

    // Open modal (click first button since empty state has two)
    const newCommandButtons = screen.getAllByRole("button", { name: /new command/i });
    await user.click(newCommandButtons[0]);

    const dialog = screen.getByRole("dialog");

    // Fill form
    await user.type(
      within(dialog).getByRole("textbox", { name: /trigger phrase/i }),
      "open spotify"
    );
    await user.type(
      within(dialog).getByRole("textbox", { name: /application/i }),
      "Spotify"
    );

    // Submit
    await user.click(
      within(dialog).getByRole("button", { name: /save command/i })
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("add_command", {
        input: {
          trigger: "open spotify",
          action_type: "open_app",
          parameters: { app: "Spotify" },
          enabled: true,
        },
      });
    });

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Command created",
      })
    );
  });

  it("validates required fields in modal", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue([]);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("No voice commands yet")).toBeDefined();
    });

    // Click first button since empty state has two
    const newCommandButtons = screen.getAllByRole("button", { name: /new command/i });
    await user.click(newCommandButtons[0]);

    const dialog = screen.getByRole("dialog");

    // Try to submit without filling required fields
    await user.click(
      within(dialog).getByRole("button", { name: /save command/i })
    );

    // Validation errors should appear
    await waitFor(() => {
      expect(screen.getByText("Trigger phrase is required")).toBeDefined();
    });
    expect(screen.getByText("Application name is required")).toBeDefined();

    // Should not have called API (only get_commands should have been called)
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("get_commands");
  });

  it("shows delete confirmation and deletes command", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(sampleCommands) // Initial load
      .mockResolvedValueOnce(undefined); // remove_command response

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    // Click delete button
    await user.click(
      screen.getByRole("button", { name: /delete open slack/i })
    );

    // Confirmation buttons should appear
    expect(screen.getByRole("button", { name: /confirm/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeDefined();

    // Confirm delete
    await user.click(screen.getByRole("button", { name: /confirm/i }));

    expect(mockInvoke).toHaveBeenCalledWith("remove_command", { id: "1" });

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Command deleted",
      })
    );
  });

  it("cancels delete when cancel button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });

    // Click delete button
    await user.click(
      screen.getByRole("button", { name: /delete open slack/i })
    );

    // Click cancel
    await user.click(screen.getByRole("button", { name: /cancel/i }));

    // Should not have called remove_command
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "remove_command",
      expect.anything()
    );

    // Edit and delete buttons should be back
    expect(
      screen.getByRole("button", { name: /edit open slack/i })
    ).toBeDefined();
  });

  it("shows advanced options in modal when expanded", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue([]);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("No voice commands yet")).toBeDefined();
    });

    // Click first button since empty state has two
    const newCommandButtons = screen.getAllByRole("button", { name: /new command/i });
    await user.click(newCommandButtons[0]);

    const dialog = screen.getByRole("dialog");

    // Advanced options should be collapsed by default
    expect(
      within(dialog).queryByRole("textbox", { name: /custom parameters/i })
    ).toBeNull();

    // Expand advanced options
    await user.click(within(dialog).getByText("Advanced Options"));

    // Advanced fields should now be visible
    expect(
      within(dialog).getByRole("textbox", { name: /custom parameters/i })
    ).toBeDefined();
    expect(
      within(dialog).getByRole("textbox", { name: /conditions/i })
    ).toBeDefined();
    expect(
      within(dialog).getByRole("checkbox", { name: /require confirmation/i })
    ).toBeDefined();
  });

  it("displays error state when loading fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Network error"));

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeDefined();
    });

    expect(screen.getByText("Network error")).toBeDefined();
    expect(screen.getByRole("button", { name: /retry/i })).toBeDefined();
  });

  it("retries loading when retry button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockRejectedValueOnce(new Error("Network error"))
      .mockResolvedValueOnce(sampleCommands);

    render(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Network error")).toBeDefined();
    });

    await user.click(screen.getByRole("button", { name: /retry/i }));

    await waitFor(() => {
      expect(screen.getByText('"open slack"')).toBeDefined();
    });
  });
});
