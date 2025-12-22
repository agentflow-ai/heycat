import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Dictionary } from "./Dictionary";
import type { DictionaryEntry } from "../types/dictionary";

// Mock Tauri invoke with vi.hoisted for proper scoping
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

// Mock toast with vi.hoisted for proper scoping
const { mockToast } = vi.hoisted(() => ({
  mockToast: vi.fn(),
}));

vi.mock("../components/overlays", () => ({
  useToast: () => ({
    toast: mockToast,
    dismiss: vi.fn(),
    dismissAll: vi.fn(),
  }),
}));

// Sample dictionary entries
const sampleEntries: DictionaryEntry[] = [
  { id: "1", trigger: "brb", expansion: "be right back" },
  { id: "2", trigger: "omw", expansion: "on my way" },
  { id: "3", trigger: "ty", expansion: "thank you" },
];

// Sample entries with settings configured
const entriesWithSettings: DictionaryEntry[] = [
  { id: "1", trigger: "brb", expansion: "be right back", suffix: ".", autoEnter: true },
  { id: "2", trigger: "omw", expansion: "on my way" },
];

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("Dictionary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("renders page with header and add form", async () => {
    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Dictionary" })).toBeDefined();
    });

    expect(
      screen.getByText("Create text expansions to speed up your typing.")
    ).toBeDefined();
    expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    expect(screen.getByLabelText("Expansion text")).toBeDefined();
    // Use getAllByRole since empty state also has an Add button
    const addButtons = screen.getAllByRole("button", { name: /add/i });
    expect(addButtons.length).toBeGreaterThan(0);
  });

  it("shows empty state when no entries exist", async () => {
    mockInvoke.mockResolvedValue([]);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("No dictionary entries yet")).toBeDefined();
    });

    expect(
      screen.getByText("Add your first text expansion to get started")
    ).toBeDefined();
  });

  it("displays entry list correctly", async () => {
    mockInvoke.mockResolvedValue(sampleEntries);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    expect(screen.getByText('"omw"')).toBeDefined();
    expect(screen.getByText('"ty"')).toBeDefined();
    expect(screen.getByText("be right back")).toBeDefined();
    expect(screen.getByText("on my way")).toBeDefined();
    expect(screen.getByText("thank you")).toBeDefined();
  });

  it("add form: submits and clears on success", async () => {
    const user = userEvent.setup();
    const newEntry: DictionaryEntry = {
      id: "new-1",
      trigger: "afk",
      expansion: "away from keyboard",
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial load
      .mockResolvedValueOnce(newEntry); // add_dictionary_entry response

    render(<Dictionary />, { wrapper: createWrapper() });

    // Wait for loading to finish (check for the form to be visible)
    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    });

    // Fill form
    await user.type(screen.getByLabelText("Trigger phrase"), "afk");
    await user.type(screen.getByLabelText("Expansion text"), "away from keyboard");

    // Submit (get by specific name from the form, not the empty state button)
    const addButtons = screen.getAllByRole("button", { name: /^add$/i });
    await user.click(addButtons[0]); // The form's Add button

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
        trigger: "afk",
        expansion: "away from keyboard",
      });
    });

    // Form should be cleared
    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toHaveValue("");
    });
    expect(screen.getByLabelText("Expansion text")).toHaveValue("");

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Entry added",
      })
    );
  });

  it("add form: shows error for empty trigger", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue([]);

    render(<Dictionary />, { wrapper: createWrapper() });

    // Wait for loading to finish
    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    });

    // Try to submit without trigger (leave trigger empty, but add expansion)
    await user.type(screen.getByLabelText("Expansion text"), "some text");

    // Submit via the form's Add button
    const addButtons = screen.getAllByRole("button", { name: /^add$/i });
    await user.click(addButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("Trigger is required")).toBeDefined();
    });

    // Should not have called add API
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "add_dictionary_entry",
      expect.anything()
    );
  });

  it("add form: shows error for duplicate trigger", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleEntries);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Try to add duplicate trigger
    await user.type(screen.getByLabelText("Trigger phrase"), "brb");
    await user.type(screen.getByLabelText("Expansion text"), "different text");

    // When entries exist, there's only the form's Add button (no empty state)
    await user.click(screen.getByRole("button", { name: /^add$/i }));

    await waitFor(() => {
      expect(screen.getByText("This trigger already exists")).toBeDefined();
    });

    // Should not have called add API
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "add_dictionary_entry",
      expect.anything()
    );
  });

  it("edit: opens edit mode and saves changes", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(sampleEntries) // Initial load
      .mockResolvedValueOnce(undefined); // update_dictionary_entry response

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Click edit button
    await user.click(screen.getByRole("button", { name: /edit brb/i }));

    // Edit mode should open with current values
    const editTriggerInput = screen.getByLabelText("Edit trigger phrase");
    expect(editTriggerInput).toHaveValue("brb");

    // Modify the expansion
    const editExpansionInput = screen.getByLabelText("Edit expansion text");
    await user.clear(editExpansionInput);
    await user.type(editExpansionInput, "be right back soon");

    // Save
    await user.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
        id: "1",
        trigger: "brb",
        expansion: "be right back soon",
      });
    });

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Entry updated",
      })
    );
  });

  it("delete: shows confirmation before deleting", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(sampleEntries) // Initial load
      .mockResolvedValueOnce(undefined); // delete_dictionary_entry response

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Click delete button
    await user.click(screen.getByRole("button", { name: /delete brb/i }));

    // Confirmation should appear
    expect(screen.getByText('Delete "brb"?')).toBeDefined();
    expect(screen.getByRole("button", { name: /confirm/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeDefined();

    // Confirm delete
    await user.click(screen.getByRole("button", { name: /confirm delete/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("delete_dictionary_entry", {
        id: "1",
      });
    });

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Entry deleted",
      })
    );
  });

  it("loading state shown while fetching", async () => {
    // Don't resolve the mock immediately
    mockInvoke.mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    render(<Dictionary />, { wrapper: createWrapper() });

    expect(screen.getByText("Loading dictionary...")).toBeDefined();
    expect(screen.getByRole("status")).toBeDefined();
  });

  it("filters entries by search query", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleEntries);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Search for "thank"
    const searchInput = screen.getByLabelText("Search dictionary entries");
    await user.type(searchInput, "thank");

    // Only matching entry should be visible
    expect(screen.queryByText('"brb"')).toBeNull();
    expect(screen.queryByText('"omw"')).toBeNull();
    expect(screen.getByText('"ty"')).toBeDefined();
    expect(screen.getByText("thank you")).toBeDefined();
  });

  it("shows no results message when search has no matches", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleEntries);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    const searchInput = screen.getByLabelText("Search dictionary entries");
    await user.type(searchInput, "nonexistent");

    expect(screen.getByText('No entries match "nonexistent"')).toBeDefined();
  });

  it("displays error state when loading fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Network error"));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeDefined();
    });

    expect(screen.getByText("Network error")).toBeDefined();
    expect(screen.getByRole("button", { name: /retry/i })).toBeDefined();
  });

  it("cancels delete when cancel button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(sampleEntries);

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Click delete button
    await user.click(screen.getByRole("button", { name: /delete brb/i }));

    // Click cancel
    await user.click(screen.getByRole("button", { name: /cancel delete/i }));

    // Should not have called delete API
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "delete_dictionary_entry",
      expect.anything()
    );

    // Edit and delete buttons should be back
    expect(screen.getByRole("button", { name: /edit brb/i })).toBeDefined();
  });

  describe("Settings Panel", () => {
    it("add form: toggles settings panel on icon click", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue([]);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Settings panel should be hidden initially
      expect(screen.queryByTestId("settings-panel")).toBeNull();

      // Click settings icon
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Settings panel should now be visible
      expect(screen.getByTestId("settings-panel")).toBeDefined();
      expect(screen.getByLabelText("Suffix")).toBeDefined();
      expect(screen.getByLabelText("Auto-enter")).toBeDefined();

      // Click settings icon again to close
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Settings panel should be hidden
      expect(screen.queryByTestId("settings-panel")).toBeNull();
    });

    it("add form: saves entry with suffix and autoEnter", async () => {
      const user = userEvent.setup();
      const newEntry: DictionaryEntry = {
        id: "new-1",
        trigger: "ty",
        expansion: "thank you",
        suffix: "!",
        autoEnter: true,
      };

      mockInvoke
        .mockResolvedValueOnce([]) // Initial load
        .mockResolvedValueOnce(newEntry); // add_dictionary_entry response

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Fill form
      await user.type(screen.getByLabelText("Trigger phrase"), "ty");
      await user.type(screen.getByLabelText("Expansion text"), "thank you");

      // Open settings
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Enter suffix
      await user.type(screen.getByLabelText("Suffix"), "!");

      // Toggle auto-enter
      await user.click(screen.getByLabelText("Auto-enter"));

      // Submit
      await user.click(screen.getByRole("button", { name: /^add$/i }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
          trigger: "ty",
          expansion: "thank you",
          suffix: "!",
          auto_enter: true,
        });
      });
    });

    it("edit: shows settings panel with correct values", async () => {
      const user = userEvent.setup();
      mockInvoke.mockReset();
      mockInvoke.mockResolvedValue(entriesWithSettings);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Click edit on entry with settings
      await user.click(screen.getByRole("button", { name: /edit brb/i }));

      // Open settings panel - get all toggle settings buttons and use the one in edit mode (not the add form one)
      const toggleButtons = screen.getAllByRole("button", { name: /toggle settings/i });
      // The first one is in add form, the second one is in edit mode
      await user.click(toggleButtons[1]);

      // Verify settings are populated
      expect(screen.getByLabelText("Suffix")).toHaveValue(".");
      expect(screen.getByLabelText("Auto-enter")).toBeChecked();
    });

    it("edit: saves entry with updated suffix and autoEnter", async () => {
      const user = userEvent.setup();
      mockInvoke.mockReset();
      mockInvoke.mockResolvedValue(entriesWithSettings);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Click edit
      await user.click(screen.getByRole("button", { name: /edit brb/i }));

      // Open settings panel - get all toggle settings buttons and use the one in edit mode (not the add form one)
      const toggleButtons = screen.getAllByRole("button", { name: /toggle settings/i });
      // The first one is in add form, the second one is in edit mode
      await user.click(toggleButtons[1]);

      // Modify suffix
      const suffixInput = screen.getByLabelText("Suffix");
      await user.clear(suffixInput);
      await user.type(suffixInput, "?");

      // Toggle auto-enter off
      await user.click(screen.getByLabelText("Auto-enter"));

      // Save
      await user.click(screen.getByRole("button", { name: /save changes/i }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
          id: "1",
          trigger: "brb",
          expansion: "be right back",
          suffix: "?",
          auto_enter: undefined, // false becomes undefined
        });
      });
    });

    it("shows settings indicator when entry has settings", async () => {
      mockInvoke.mockReset();
      mockInvoke.mockResolvedValue(entriesWithSettings);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Entry with settings should have a visible settings indicator
      // The Settings icon within the entry row indicates settings are configured
      const entryList = screen.getByRole("list");
      const entries = entryList.querySelectorAll('[role="listitem"]');

      // First entry (brb) has settings - should have heycat-orange colored icon
      const firstEntry = entries[0];
      const settingsIndicator = firstEntry.querySelector(".text-heycat-orange");
      expect(settingsIndicator).not.toBeNull();

      // Second entry (omw) has no settings - should not have indicator
      const secondEntry = entries[1];
      const noIndicator = secondEntry.querySelector(".text-heycat-orange");
      expect(noIndicator).toBeNull();
    });
  });

  describe("Suffix Validation", () => {
    it("add form: allows exactly 5 characters without error", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue([]);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Open settings panel
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Type exactly 5 characters
      await user.type(screen.getByLabelText("Suffix"), "12345");

      // No error should be shown
      expect(screen.queryByText("Suffix must be 5 characters or less")).toBeNull();

      // Add button should be enabled
      const addButton = screen.getByRole("button", { name: /^add$/i });
      expect(addButton).not.toBeDisabled();
    });

    it("add form: shows error and disables save when suffix exceeds 5 characters via paste", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue([]);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Fill in required fields first
      await user.type(screen.getByLabelText("Trigger phrase"), "test");
      await user.type(screen.getByLabelText("Expansion text"), "test expansion");

      // Open settings panel
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Simulate paste (bypass maxLength) using fireEvent
      const suffixInput = screen.getByLabelText("Suffix");
      // Paste bypasses maxLength so we use fireEvent
      fireEvent.change(suffixInput, { target: { value: "123456" } });

      // Error should be shown
      expect(screen.getByText("Suffix must be 5 characters or less")).toBeDefined();

      // Add button should be disabled
      const addButton = screen.getByRole("button", { name: /^add$/i });
      expect(addButton).toBeDisabled();
    });

    it("add form: clears error when suffix is corrected to 5 or fewer characters", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue([]);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Open settings panel
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Set invalid value (bypass maxLength via fireEvent)
      const suffixInput = screen.getByLabelText("Suffix");
      fireEvent.change(suffixInput, { target: { value: "123456" } });

      // Error should be shown
      expect(screen.getByText("Suffix must be 5 characters or less")).toBeDefined();

      // Correct to valid value
      fireEvent.change(suffixInput, { target: { value: "12345" } });

      // Error should be cleared
      expect(screen.queryByText("Suffix must be 5 characters or less")).toBeNull();

      // Add button should be enabled
      const addButton = screen.getByRole("button", { name: /^add$/i });
      expect(addButton).not.toBeDisabled();
    });

    it("add form: allows empty suffix without error", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue([]);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
      });

      // Open settings panel
      await user.click(screen.getByRole("button", { name: /toggle settings/i }));

      // Suffix is empty by default - no error should be shown
      expect(screen.queryByText("Suffix must be 5 characters or less")).toBeNull();

      // Add button should be enabled
      const addButton = screen.getByRole("button", { name: /^add$/i });
      expect(addButton).not.toBeDisabled();
    });

    it("edit: shows error and disables save when suffix exceeds 5 characters", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(sampleEntries);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Click edit button
      await user.click(screen.getByRole("button", { name: /edit brb/i }));

      // Open settings panel in edit mode
      const toggleButtons = screen.getAllByRole("button", { name: /toggle settings/i });
      await user.click(toggleButtons[1]); // Edit mode toggle

      // Set invalid suffix value (bypass maxLength via fireEvent)
      const suffixInput = screen.getByLabelText("Suffix");
      fireEvent.change(suffixInput, { target: { value: "123456" } });

      // Error should be shown
      expect(screen.getByText("Suffix must be 5 characters or less")).toBeDefined();

      // Save button should be disabled
      const saveButton = screen.getByRole("button", { name: /save changes/i });
      expect(saveButton).toBeDisabled();
    });

    it("edit: clears error when suffix is corrected", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(sampleEntries);

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Click edit button
      await user.click(screen.getByRole("button", { name: /edit brb/i }));

      // Open settings panel in edit mode
      const toggleButtons = screen.getAllByRole("button", { name: /toggle settings/i });
      await user.click(toggleButtons[1]); // Edit mode toggle

      // Set invalid suffix value
      const suffixInput = screen.getByLabelText("Suffix");
      fireEvent.change(suffixInput, { target: { value: "123456" } });

      // Error should be shown
      expect(screen.getByText("Suffix must be 5 characters or less")).toBeDefined();

      // Correct to valid value
      fireEvent.change(suffixInput, { target: { value: "12345" } });

      // Error should be cleared
      expect(screen.queryByText("Suffix must be 5 characters or less")).toBeNull();

      // Save button should be enabled
      const saveButton = screen.getByRole("button", { name: /save changes/i });
      expect(saveButton).not.toBeDisabled();
    });
  });
});
