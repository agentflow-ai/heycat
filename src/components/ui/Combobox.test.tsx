import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { Combobox, type ComboboxOption } from "./Combobox";

const mockOptions: ComboboxOption[] = [
  { label: "Safari", value: "Safari", description: "com.apple.Safari" },
  { label: "Finder", value: "Finder", description: "com.apple.finder" },
  { label: "Slack", value: "Slack", description: "com.tinyspeck.slackmacgap" },
  { label: "Visual Studio Code", value: "Visual Studio Code", description: "com.microsoft.VSCode" },
];

describe("Combobox", () => {
  it("renders input with placeholder", () => {
    render(
      <Combobox
        value=""
        onChange={() => {}}
        options={mockOptions}
        placeholder="Select an app"
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute("placeholder", "Select an app");
  });

  it("shows all options when input is focused and empty", async () => {
    const user = userEvent.setup();
    render(
      <Combobox
        value=""
        onChange={() => {}}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // All options should be visible
    expect(screen.getByText("Safari")).toBeInTheDocument();
    expect(screen.getByText("Finder")).toBeInTheDocument();
    expect(screen.getByText("Slack")).toBeInTheDocument();
    expect(screen.getByText("Visual Studio Code")).toBeInTheDocument();
  });

  it("filters options based on typed text", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <Combobox
        value="Sla"
        onChange={() => {}}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Only Slack should match "Sla"
    expect(screen.getByText("Slack")).toBeInTheDocument();
    expect(screen.queryByText("Safari")).not.toBeInTheDocument();
    expect(screen.queryByText("Finder")).not.toBeInTheDocument();
  });

  it("calls onSelect with full option data when selecting a suggestion", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    const onSelect = vi.fn();

    render(
      <Combobox
        value=""
        onChange={onChange}
        onSelect={onSelect}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Click on Slack option
    const slackOption = screen.getByText("Slack");
    await user.click(slackOption);

    // onSelect should be called with full option data
    expect(onSelect).toHaveBeenCalledWith({
      label: "Slack",
      value: "Slack",
      description: "com.tinyspeck.slackmacgap",
    });

    // onChange should also be called with the value
    expect(onChange).toHaveBeenCalledWith("Slack");
  });

  it("allows custom text that doesn't match any suggestion", async () => {
    const user = userEvent.setup();
    render(
      <Combobox
        value="CustomApp"
        onChange={() => {}}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Should show "no matching applications" message
    expect(screen.getByText(/no matching applications/i)).toBeInTheDocument();
  });

  it("displays bundle ID as description in options", async () => {
    const user = userEvent.setup();
    render(
      <Combobox
        value=""
        onChange={() => {}}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Bundle IDs should be visible as descriptions
    expect(screen.getByText("com.apple.Safari")).toBeInTheDocument();
    expect(screen.getByText("com.apple.finder")).toBeInTheDocument();
    expect(screen.getByText("com.tinyspeck.slackmacgap")).toBeInTheDocument();
  });

  it("navigates options with keyboard", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();

    render(
      <Combobox
        value=""
        onChange={() => {}}
        onSelect={onSelect}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Press ArrowDown to highlight first option
    await user.keyboard("{ArrowDown}");

    // Press Enter to select
    await user.keyboard("{Enter}");

    // First option (Safari - first in the mockOptions array) should be selected
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ value: "Safari" }));
  });

  it("closes dropdown on Escape", async () => {
    const user = userEvent.setup();
    render(
      <Combobox
        value=""
        onChange={() => {}}
        options={mockOptions}
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Options should be visible
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
        <Combobox
          value=""
          onChange={() => {}}
          options={mockOptions}
          aria-label="App name"
        />
        <button>Outside button</button>
      </div>
    );

    const input = screen.getByRole("combobox");
    await user.click(input);

    // Options should be visible
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
      <Combobox
        value=""
        onChange={() => {}}
        options={mockOptions}
        disabled
        aria-label="App name"
      />
    );

    const input = screen.getByRole("combobox");
    expect(input).toBeDisabled();
  });
});
