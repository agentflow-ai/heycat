import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { MultiSelect, type MultiSelectOption } from "./MultiSelect";

const mockOptions: MultiSelectOption[] = [
  { value: "dict-1", label: "btw", description: "by the way" },
  { value: "dict-2", label: "brb", description: "be right back" },
  { value: "dict-3", label: "lol", description: "laugh out loud" },
  { value: "dict-4", label: "omg", description: "oh my god" },
];

describe("MultiSelect", () => {
  it("renders input with placeholder when nothing is selected", () => {
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        placeholder="Select items..."
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    expect(input).toHaveAttribute("placeholder", "Select items...");
  });

  it("shows all options when input is focused", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // All options should be visible
    expect(screen.getByText("btw")).toBeInTheDocument();
    expect(screen.getByText("brb")).toBeInTheDocument();
    expect(screen.getByText("lol")).toBeInTheDocument();
    expect(screen.getByText("omg")).toBeInTheDocument();
  });

  it("displays descriptions for options", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Descriptions should be visible
    expect(screen.getByText("by the way")).toBeInTheDocument();
    expect(screen.getByText("be right back")).toBeInTheDocument();
  });

  it("adds item to selection when clicking an option", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <MultiSelect
        selected={[]}
        onChange={onChange}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Click on "btw" option
    const btwOption = screen.getByText("btw");
    await user.click(btwOption);

    // onChange should be called with the selected value
    expect(onChange).toHaveBeenCalledWith(["dict-1"]);
  });

  it("removes item from selection when clicking a selected option", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <MultiSelect
        selected={["dict-1", "dict-2"]}
        onChange={onChange}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Click on "btw" option (already selected) - use the option role to disambiguate
    const btwOption = screen.getByRole("option", { name: /btw/i });
    await user.click(btwOption);

    // onChange should be called without the deselected value
    expect(onChange).toHaveBeenCalledWith(["dict-2"]);
  });

  it("shows selected items as tags", () => {
    render(
      <MultiSelect
        selected={["dict-1", "dict-2"]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    // Selected items should appear as tags (outside the dropdown)
    const tags = screen.getAllByText(/btw|brb/);
    expect(tags.length).toBeGreaterThanOrEqual(1);
  });

  it("removes item when clicking the X button on a tag", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <MultiSelect
        selected={["dict-1"]}
        onChange={onChange}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    // Click the remove button on the tag
    const removeButton = screen.getByRole("button", { name: /remove btw/i });
    await user.click(removeButton);

    // onChange should be called without the removed value
    expect(onChange).toHaveBeenCalledWith([]);
  });

  it("filters options based on search term", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);
    await user.type(input, "br");

    // Only "brb" should match the search
    expect(screen.getByText("brb")).toBeInTheDocument();
    expect(screen.queryByText("btw")).not.toBeInTheDocument();
    expect(screen.queryByText("lol")).not.toBeInTheDocument();
  });

  it("shows empty message when no options match search", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);
    await user.type(input, "xyz");

    // Should show no matching options message
    expect(screen.getByText("No matching options")).toBeInTheDocument();
  });

  it("shows empty message when no options are available", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={[]}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Should show no options available message
    expect(screen.getByText("No options available")).toBeInTheDocument();
  });

  it("navigates options with keyboard", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <MultiSelect
        selected={[]}
        onChange={onChange}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Press ArrowDown to highlight first option
    await user.keyboard("{ArrowDown}");

    // Press Enter to select
    await user.keyboard("{Enter}");

    // First option should be selected
    expect(onChange).toHaveBeenCalledWith(["dict-1"]);
  });

  it("closes dropdown on Escape", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Dropdown should be visible
    expect(screen.getByRole("listbox")).toBeInTheDocument();

    // Press Escape
    await user.keyboard("{Escape}");

    // Dropdown should close
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });

  it("closes dropdown when clicking outside", async () => {
    const user = userEvent.setup();
    render(
      <div>
        <MultiSelect
          selected={[]}
          onChange={() => {}}
          options={mockOptions}
          aria-label="Dictionary entries"
        />
        <button>Outside button</button>
      </div>
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Dropdown should be visible
    expect(screen.getByRole("listbox")).toBeInTheDocument();

    // Click outside
    const outsideButton = screen.getByText("Outside button");
    await user.click(outsideButton);

    // Dropdown should close
    await waitFor(() => {
      expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    });
  });

  it("respects disabled state", () => {
    render(
      <MultiSelect
        selected={[]}
        onChange={() => {}}
        options={mockOptions}
        disabled
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    expect(input).toBeDisabled();
  });

  it("shows checkmarks for selected items in dropdown", async () => {
    const user = userEvent.setup();
    render(
      <MultiSelect
        selected={["dict-1"]}
        onChange={() => {}}
        options={mockOptions}
        aria-label="Dictionary entries"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // The "btw" option should be marked as selected
    const btwOption = screen.getByRole("option", { name: /btw/i });
    expect(btwOption).toHaveAttribute("aria-selected", "true");
  });
});
