import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CommandSettings } from "./CommandSettings";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("CommandSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Empty state", () => {
    it("displays empty state when no commands exist", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("No commands configured")).toBeDefined();
      });
      expect(
        screen.getByText("Add your first voice command to get started")
      ).toBeDefined();
    });
  });

  describe("Command list", () => {
    const mockCommands = [
      {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      },
      {
        id: "cmd-2",
        trigger: "type hello",
        action_type: "type_text",
        parameters: { text: "Hello World" },
        enabled: false,
      },
    ];

    it("displays all commands with trigger and action type", async () => {
      mockInvoke.mockResolvedValueOnce(mockCommands);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });
      expect(screen.getByText("Open App")).toBeDefined();
      expect(screen.getByText("type hello")).toBeDefined();
      expect(screen.getByText("Type Text")).toBeDefined();
    });

    it("shows enabled status correctly", async () => {
      mockInvoke.mockResolvedValueOnce(mockCommands);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      const toggles = screen.getAllByRole("checkbox");
      expect(toggles[0]).toHaveProperty("checked", true);
      expect(toggles[1]).toHaveProperty("checked", false);
    });
  });

  describe("Adding commands", () => {
    it("shows editor when Add Command is clicked", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));

      expect(screen.getByText("Trigger Phrase")).toBeDefined();
      expect(screen.getByText("Action Type")).toBeDefined();
    });

    it("adds command and shows it in the list", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));

      const triggerInput = screen.getByLabelText("Trigger Phrase");
      await userEvent.type(triggerInput, "launch safari");

      const appInput = screen.getByLabelText("App Name");
      await userEvent.type(appInput, "Safari");

      const newCommand = {
        id: "new-cmd",
        trigger: "launch safari",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce(newCommand);

      await userEvent.click(
        screen.getByRole("button", { name: "Add Command" })
      );

      await waitFor(() => {
        expect(screen.getByText("launch safari")).toBeDefined();
      });
    });
  });

  describe("Editing commands", () => {
    it("opens editor with command data when Edit is clicked", async () => {
      const mockCommand = {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce([mockCommand]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      await userEvent.click(screen.getByLabelText("Edit open browser"));

      expect(screen.getByText("Edit Command")).toBeDefined();
      expect(screen.getByDisplayValue("open browser")).toBeDefined();
      expect(screen.getByDisplayValue("Safari")).toBeDefined();
    });

    it("updates command trigger in the list", async () => {
      const mockCommand = {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce([mockCommand]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      await userEvent.click(screen.getByLabelText("Edit open browser"));

      const triggerInput = screen.getByLabelText("Trigger Phrase");
      await userEvent.clear(triggerInput);
      await userEvent.type(triggerInput, "launch browser");

      // Mock remove and add for edit
      mockInvoke.mockResolvedValueOnce(undefined); // remove_command
      mockInvoke.mockResolvedValueOnce({
        ...mockCommand,
        id: "cmd-1-updated",
        trigger: "launch browser",
      });

      await userEvent.click(screen.getByText("Save Changes"));

      await waitFor(() => {
        expect(screen.getByText("launch browser")).toBeDefined();
      });
    });
  });

  describe("Deleting commands", () => {
    it("shows confirmation when Delete is clicked", async () => {
      const mockCommand = {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce([mockCommand]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      await userEvent.click(screen.getByLabelText("Delete open browser"));

      expect(screen.getByLabelText("Confirm delete")).toBeDefined();
      expect(screen.getByLabelText("Cancel delete")).toBeDefined();
    });

    it("removes command after confirmation", async () => {
      const mockCommand = {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce([mockCommand]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      await userEvent.click(screen.getByLabelText("Delete open browser"));
      mockInvoke.mockResolvedValueOnce(undefined);
      await userEvent.click(screen.getByLabelText("Confirm delete"));

      await waitFor(() => {
        expect(screen.getByText("No commands configured")).toBeDefined();
      });
    });
  });

  describe("Form validation", () => {
    it("shows error for duplicate trigger", async () => {
      const mockCommand = {
        id: "cmd-1",
        trigger: "open browser",
        action_type: "open_app",
        parameters: { app: "Safari" },
        enabled: true,
      };
      mockInvoke.mockResolvedValueOnce([mockCommand]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("open browser")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));

      const triggerInput = screen.getByLabelText("Trigger Phrase");
      await userEvent.type(triggerInput, "open browser");

      const appInput = screen.getByLabelText("App Name");
      await userEvent.type(appInput, "Chrome");

      await userEvent.click(
        screen.getByRole("button", { name: "Add Command" })
      );

      expect(screen.getByText("Trigger already exists")).toBeDefined();
    });

    it("shows error for empty required fields", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));
      await userEvent.click(
        screen.getByRole("button", { name: "Add Command" })
      );

      expect(screen.getByText("Trigger is required")).toBeDefined();
      expect(screen.getByText("App name is required")).toBeDefined();
    });
  });

  describe("Parameter fields by action type", () => {
    it("shows app input for open_app action", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));

      expect(screen.getByLabelText("App Name")).toBeDefined();
    });

    it("shows text and delay inputs for type_text action", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));
      await userEvent.selectOptions(
        screen.getByLabelText("Action Type"),
        "type_text"
      );

      expect(screen.getByLabelText("Text to Type")).toBeDefined();
      expect(screen.getByLabelText("Delay (ms)")).toBeDefined();
    });

    it("shows control dropdown for system_control action", async () => {
      mockInvoke.mockResolvedValueOnce([]);

      render(<CommandSettings />);

      await waitFor(() => {
        expect(screen.getByText("Add Command")).toBeDefined();
      });

      await userEvent.click(screen.getByText("Add Command"));
      await userEvent.selectOptions(
        screen.getByLabelText("Action Type"),
        "system_control"
      );

      expect(screen.getByLabelText("Control Type")).toBeDefined();
    });
  });
});
