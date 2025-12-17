import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Input, Textarea, Label, FormField } from "./Input";

describe("Input", () => {
  it("renders with correct base styles", () => {
    render(<Input data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.className).toContain("bg-white");
    expect(input.className).toContain("border");
    expect(input.className).toContain("text-base");
  });

  it("has teal focus ring on focus", () => {
    render(<Input data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.className).toContain("focus:border-heycat-teal");
    expect(input.className).toContain("focus:ring-2");
  });

  it("shows placeholder with correct styling", () => {
    render(<Input placeholder="Enter text..." data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.className).toContain("placeholder:text-neutral-400");
  });

  it("shows error state with red border", () => {
    render(<Input error data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.className).toContain("border-error");
  });

  it("handles disabled state", () => {
    render(<Input disabled data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.hasAttribute("disabled")).toBe(true);
    expect(input.className).toContain("disabled:opacity-50");
    expect(input.className).toContain("disabled:cursor-not-allowed");
  });

  it("accepts user input", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<Input onChange={handleChange} data-testid="input" />);

    await user.type(screen.getByTestId("input"), "Hello");
    expect(handleChange).toHaveBeenCalled();
  });

  it("merges custom className", () => {
    render(<Input className="custom-class" data-testid="input" />);
    const input = screen.getByTestId("input");
    expect(input.className).toContain("custom-class");
    expect(input.className).toContain("bg-white");
  });
});

describe("Textarea", () => {
  it("renders with correct base styles", () => {
    render(<Textarea data-testid="textarea" />);
    const textarea = screen.getByTestId("textarea");
    expect(textarea.tagName).toBe("TEXTAREA");
    expect(textarea.className).toContain("bg-white");
    expect(textarea.className).toContain("resize-y");
  });

  it("has teal focus ring on focus", () => {
    render(<Textarea data-testid="textarea" />);
    const textarea = screen.getByTestId("textarea");
    expect(textarea.className).toContain("focus:border-heycat-teal");
  });

  it("shows error state with red border", () => {
    render(<Textarea error data-testid="textarea" />);
    const textarea = screen.getByTestId("textarea");
    expect(textarea.className).toContain("border-error");
  });
});

describe("Label", () => {
  it("renders label text", () => {
    render(<Label>Email</Label>);
    expect(screen.getByText("Email")).toBeDefined();
  });

  it("shows required indicator", () => {
    render(<Label required>Email</Label>);
    expect(screen.getByText("*")).toBeDefined();
  });

  it("has correct typography styles", () => {
    render(<Label data-testid="label">Email</Label>);
    const label = screen.getByTestId("label");
    expect(label.className).toContain("text-sm");
    expect(label.className).toContain("font-medium");
  });

  it("renders as label element", () => {
    render(<Label data-testid="label">Email</Label>);
    expect(screen.getByTestId("label").tagName).toBe("LABEL");
  });
});

describe("FormField", () => {
  it("renders children", () => {
    render(
      <FormField data-testid="field">
        <Input data-testid="input" />
      </FormField>
    );
    expect(screen.getByTestId("field")).toBeDefined();
    expect(screen.getByTestId("input")).toBeDefined();
  });

  it("shows error message when error prop is set", () => {
    render(
      <FormField error="This field is required" data-testid="field">
        <Input data-testid="input" />
      </FormField>
    );
    expect(screen.getByText("This field is required")).toBeDefined();
    expect(screen.getByRole("alert").textContent).toBe("This field is required");
  });

  it("has margin bottom for spacing", () => {
    render(
      <FormField data-testid="field">
        <Input />
      </FormField>
    );
    expect(screen.getByTestId("field").className).toContain("mb-4");
  });
});
