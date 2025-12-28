import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Dictionary } from "./Dictionary";
import type { DictionaryEntry } from "../types/dictionary";
import type { WindowContext } from "../types/windowContext";

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

// Sample window contexts for testing context badges
const sampleContexts: WindowContext[] = [
  {
    id: "ctx-1",
    name: "Slack",
    matcher: { appName: "Slack" },
    enabled: true,
    priority: 0,
    commandMode: "merge",
    dictionaryMode: "merge",
    commandIds: [],
    dictionaryEntryIds: ["1"], // Entry "brb" is assigned to Slack
  },
  {
    id: "ctx-2",
    name: "VS Code",
    matcher: { appName: "Code" },
    enabled: true,
    priority: 0,
    commandMode: "merge",
    dictionaryMode: "merge",
    commandIds: [],
    dictionaryEntryIds: ["1", "2"], // Both "brb" and "omw" assigned to VS Code
  },
  {
    id: "ctx-3",
    name: "Terminal",
    matcher: { appName: "Terminal" },
    enabled: true,
    priority: 0,
    commandMode: "merge",
    dictionaryMode: "merge",
    commandIds: [],
    dictionaryEntryIds: ["1"], // Entry "brb" also assigned to Terminal
  },
];

// Helper to create invoke mock that routes commands correctly
function createInvokeMock(
  entries: DictionaryEntry[],
  contexts: WindowContext[] = [],
  extraHandlers: Record<string, unknown> = {}
) {
  return (command: string, _args?: unknown) => {
    if (command === "list_dictionary_entries") {
      return Promise.resolve(entries);
    }
    if (command === "list_window_contexts") {
      return Promise.resolve(contexts);
    }
    if (command in extraHandlers) {
      return Promise.resolve(extraHandlers[command]);
    }
    return Promise.resolve(undefined);
  };
}

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
    // Default mock: empty entries, empty contexts
    mockInvoke.mockImplementation(createInvokeMock([], []));
  });

  it("displays entry list correctly", async () => {
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

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

  it("add form: submits with settings and clears on success", async () => {
    const user = userEvent.setup();
    const newEntry: DictionaryEntry = {
      id: "new-1",
      trigger: "afk",
      expansion: "away from keyboard",
      suffix: "!",
      autoEnter: true,
    };

    mockInvoke.mockImplementation(createInvokeMock([], [], { add_dictionary_entry: newEntry }));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    });

    // Fill form using fireEvent for speed
    fireEvent.change(screen.getByLabelText("Trigger phrase"), { target: { value: "afk" } });
    fireEvent.change(screen.getByLabelText("Expansion text"), { target: { value: "away from keyboard" } });

    // Open settings and configure
    await user.click(screen.getByRole("button", { name: /toggle settings/i }));
    fireEvent.change(screen.getByLabelText("Suffix"), { target: { value: "!" } });
    await user.click(screen.getByLabelText("Auto-enter"));

    // Submit
    await user.click(screen.getByRole("button", { name: /^add$/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
        trigger: "afk",
        expansion: "away from keyboard",
        suffix: "!",
        autoEnter: true,
      });
    });

    // Form should be cleared
    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toHaveValue("");
    });

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
    // Uses default mock from beforeEach

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    });

    // Add expansion but leave trigger empty
    fireEvent.change(screen.getByLabelText("Expansion text"), { target: { value: "some text" } });
    await user.click(screen.getByRole("button", { name: /^add$/i }));

    await waitFor(() => {
      expect(screen.getByText("Trigger is required")).toBeDefined();
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("add_dictionary_entry", expect.anything());
  });

  it("add form: shows error for duplicate trigger", async () => {
    const user = userEvent.setup();
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Try to add duplicate trigger
    fireEvent.change(screen.getByLabelText("Trigger phrase"), { target: { value: "brb" } });
    fireEvent.change(screen.getByLabelText("Expansion text"), { target: { value: "different text" } });
    await user.click(screen.getByRole("button", { name: /^add$/i }));

    await waitFor(() => {
      expect(screen.getByText("This trigger already exists")).toBeDefined();
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("add_dictionary_entry", expect.anything());
  });

  it("edit: opens, shows settings, and saves changes", async () => {
    const user = userEvent.setup();
    mockInvoke.mockImplementation(createInvokeMock(entriesWithSettings, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Click edit button
    await user.click(screen.getByRole("button", { name: /edit brb/i }));

    // Edit mode should open with current values
    expect(screen.getByLabelText("Edit trigger phrase")).toHaveValue("brb");

    // Open settings panel and verify values
    const toggleButtons = screen.getAllByRole("button", { name: /toggle settings/i });
    await user.click(toggleButtons[1]); // Edit mode toggle
    expect(screen.getByLabelText("Suffix")).toHaveValue(".");
    expect(screen.getByLabelText("Auto-enter")).toBeChecked();

    // Modify expansion and settings
    fireEvent.change(screen.getByLabelText("Edit expansion text"), { target: { value: "be right back soon" } });
    fireEvent.change(screen.getByLabelText("Suffix"), { target: { value: "?" } });
    await user.click(screen.getByLabelText("Auto-enter")); // Toggle off

    // Save
    await user.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
        id: "1",
        trigger: "brb",
        expansion: "be right back soon",
        suffix: "?",
        autoEnter: undefined,
      });
    });

    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Entry updated",
      })
    );
  });

  it("delete: shows confirmation and deletes", async () => {
    const user = userEvent.setup();
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Click delete button
    await user.click(screen.getByRole("button", { name: /delete brb/i }));

    // Confirmation should appear
    expect(screen.getByText('Delete "brb"?')).toBeDefined();

    // Confirm delete
    await user.click(screen.getByRole("button", { name: /confirm delete/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("delete_dictionary_entry", { id: "1" });
    });

    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Entry deleted",
      })
    );
  });

  it("cancels delete when cancel clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    await user.click(screen.getByRole("button", { name: /delete brb/i }));
    await user.click(screen.getByRole("button", { name: /cancel delete/i }));

    expect(mockInvoke).not.toHaveBeenCalledWith("delete_dictionary_entry", expect.anything());
    expect(screen.getByRole("button", { name: /edit brb/i })).toBeDefined();
  });

  it("filters entries by search query", async () => {
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    // Search for "thank"
    fireEvent.change(screen.getByLabelText("Search dictionary entries"), { target: { value: "thank" } });

    // Only matching entry should be visible
    expect(screen.queryByText('"brb"')).toBeNull();
    expect(screen.queryByText('"omw"')).toBeNull();
    expect(screen.getByText('"ty"')).toBeDefined();
    expect(screen.getByText("thank you")).toBeDefined();
  });

  it("shows no results when search has no matches", async () => {
    mockInvoke.mockImplementation(createInvokeMock(sampleEntries, []));

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('"brb"')).toBeDefined();
    });

    fireEvent.change(screen.getByLabelText("Search dictionary entries"), { target: { value: "nonexistent" } });

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

  it("suffix validation: shows error for >5 chars and clears when corrected", async () => {
    const user = userEvent.setup();
    // Uses default mock from beforeEach

    render(<Dictionary />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByLabelText("Trigger phrase")).toBeDefined();
    });

    // Fill required fields
    fireEvent.change(screen.getByLabelText("Trigger phrase"), { target: { value: "test" } });
    fireEvent.change(screen.getByLabelText("Expansion text"), { target: { value: "test expansion" } });

    // Open settings
    await user.click(screen.getByRole("button", { name: /toggle settings/i }));

    // Set invalid suffix (>5 chars via paste bypass)
    fireEvent.change(screen.getByLabelText("Suffix"), { target: { value: "123456" } });

    // Error should be shown and button disabled
    expect(screen.getByText("Suffix must be 5 characters or less")).toBeDefined();
    expect(screen.getByRole("button", { name: /^add$/i })).toBeDisabled();

    // Correct to valid value
    fireEvent.change(screen.getByLabelText("Suffix"), { target: { value: "." } });

    // Error should be cleared and button enabled
    expect(screen.queryByText("Suffix must be 5 characters or less")).toBeNull();
    expect(screen.getByRole("button", { name: /^add$/i })).not.toBeDisabled();
  });

  describe("Context Badges", () => {
    it("shows 'Global' badge for entry with no context assignments", async () => {
      // Entry "ty" (id: 3) is not assigned to any context
      mockInvoke.mockImplementation(createInvokeMock(sampleEntries, sampleContexts));

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"ty"')).toBeDefined();
      });

      // Find the Global badge for "ty" entry
      const globalBadges = screen.getAllByTestId("context-badge-global");
      expect(globalBadges.length).toBeGreaterThanOrEqual(1);
      expect(globalBadges[0]).toHaveTextContent("Global");
    });

    it("shows context name badge for entry with 1 context", async () => {
      // Entry "omw" (id: 2) is assigned to only "VS Code"
      mockInvoke.mockImplementation(createInvokeMock(sampleEntries, sampleContexts));

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"omw"')).toBeDefined();
      });

      // Should show "VS Code" badge
      expect(screen.getByText("VS Code")).toBeDefined();
    });

    it("shows two context name badges for entry with 2 contexts", async () => {
      // Create a scenario where "omw" is assigned to exactly 2 contexts
      const twoContexts: WindowContext[] = [
        {
          id: "ctx-1",
          name: "Slack",
          matcher: { appName: "Slack" },
          enabled: true,
          priority: 0,
          commandMode: "merge",
          dictionaryMode: "merge",
          commandIds: [],
          dictionaryEntryIds: ["2"], // Only "omw"
        },
        {
          id: "ctx-2",
          name: "VS Code",
          matcher: { appName: "Code" },
          enabled: true,
          priority: 0,
          commandMode: "merge",
          dictionaryMode: "merge",
          commandIds: [],
          dictionaryEntryIds: ["2"], // Only "omw"
        },
      ];

      mockInvoke.mockImplementation(createInvokeMock(sampleEntries, twoContexts));

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"omw"')).toBeDefined();
      });

      // Both context names should appear as separate badges
      const contextBadges = screen.getAllByTestId("context-badge");
      expect(contextBadges.length).toBe(2);
      expect(screen.getByText("Slack")).toBeDefined();
      expect(screen.getByText("VS Code")).toBeDefined();
    });

    it("shows 'N contexts' badge for entry with 3+ contexts", async () => {
      // Entry "brb" (id: 1) is assigned to Slack, VS Code, and Terminal (3 contexts)
      mockInvoke.mockImplementation(createInvokeMock(sampleEntries, sampleContexts));

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Should show "3 contexts" badge
      const countBadge = screen.getByTestId("context-badge-count");
      expect(countBadge).toHaveTextContent("3 contexts");
    });

    it("count badge shows tooltip with all context names", async () => {
      // Entry "brb" (id: 1) is assigned to Slack, VS Code, and Terminal
      mockInvoke.mockImplementation(createInvokeMock(sampleEntries, sampleContexts));

      render(<Dictionary />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText('"brb"')).toBeDefined();
      });

      // Badge should have title attribute with all context names
      const countBadge = screen.getByTestId("context-badge-count");
      expect(countBadge).toHaveAttribute("title", "Slack, VS Code, Terminal");
    });
  });
});
