import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Toggle, LabeledToggle } from "./Toggle";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Toggle", () => {
  it("toggles between on and off states when clicked", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<Toggle onCheckedChange={handleChange} />);

    const toggle = screen.getByRole("switch");
    expect(toggle.getAttribute("aria-checked")).toBe("false");

    await user.click(toggle);
    expect(handleChange).toHaveBeenCalledWith(true);
    expect(toggle.getAttribute("aria-checked")).toBe("true");
  });

  it("respects controlled checked prop", () => {
    render(<Toggle checked={true} />);
    const toggle = screen.getByRole("switch");
    expect(toggle.getAttribute("aria-checked")).toBe("true");
  });

  it("cannot be toggled when disabled", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<Toggle disabled onCheckedChange={handleChange} />);

    await user.click(screen.getByRole("switch"));
    expect(handleChange).not.toHaveBeenCalled();
  });
});

describe("LabeledToggle", () => {
  it("clicking the label toggles the switch", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<LabeledToggle label="Enable feature" onCheckedChange={handleChange} />);

    await user.click(screen.getByText("Enable feature"));
    expect(handleChange).toHaveBeenCalledWith(true);
  });

  it("shows description text when provided", () => {
    render(
      <LabeledToggle
        label="Notifications"
        description="Get alerts for new recordings"
      />
    );
    expect(screen.getByText("Get alerts for new recordings")).toBeDefined();
  });
});
