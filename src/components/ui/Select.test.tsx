import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Select, SelectItem, SelectGroup, SelectSeparator } from "./Select";

// Note: Radix UI Select uses pointer capture APIs that jsdom doesn't fully support.
// These tests focus on initial render and accessibility attributes.
// Interactive behavior should be verified with E2E tests.

describe("Select", () => {
  it("renders with placeholder", () => {
    render(
      <Select placeholder="Select an option">
        <SelectItem value="option1">Option 1</SelectItem>
      </Select>
    );
    expect(screen.getByText("Select an option")).toBeDefined();
  });

  it("renders trigger with correct styles", () => {
    render(
      <Select placeholder="Select...">
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    const trigger = screen.getByRole("combobox");
    expect(trigger.className).toContain("bg-white");
    expect(trigger.className).toContain("border");
    expect(trigger.className).toContain("focus:border-heycat-teal");
  });

  it("has chevron icon in trigger", () => {
    render(
      <Select placeholder="Select...">
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    const trigger = screen.getByRole("combobox");
    const svg = trigger.querySelector("svg");
    expect(svg).not.toBeNull();
  });

  it("renders trigger with combobox role", () => {
    render(
      <Select placeholder="Select...">
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("shows default value in trigger", () => {
    render(
      <Select defaultValue="option1">
        <SelectItem value="option1">Option 1</SelectItem>
        <SelectItem value="option2">Option 2</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox").textContent).toContain("Option 1");
  });

  it("shows controlled value in trigger", () => {
    render(
      <Select value="option2">
        <SelectItem value="option1">Option 1</SelectItem>
        <SelectItem value="option2">Option 2</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox").textContent).toContain("Option 2");
  });

  it("applies disabled attribute when disabled", () => {
    render(
      <Select placeholder="Select..." disabled>
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    const trigger = screen.getByRole("combobox");
    expect(trigger.getAttribute("data-disabled")).toBe("");
  });

  it("has focus ring classes for accessibility", () => {
    render(
      <Select placeholder="Select...">
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    const trigger = screen.getByRole("combobox");
    expect(trigger.className).toContain("focus:ring-2");
    expect(trigger.className).toContain("focus:outline-none");
  });
});

describe("Select exports", () => {
  it("exports SelectItem component", () => {
    expect(SelectItem).toBeDefined();
  });

  it("exports SelectGroup component", () => {
    expect(SelectGroup).toBeDefined();
  });

  it("exports SelectSeparator component", () => {
    expect(SelectSeparator).toBeDefined();
  });
});
