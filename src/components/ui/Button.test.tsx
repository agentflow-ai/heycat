import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

describe("Button", () => {
  describe("variants", () => {
    it("renders primary variant with gradient background", () => {
      render(<Button variant="primary">Primary</Button>);
      const button = screen.getByRole("button", { name: "Primary" });
      expect(button.className).toContain("bg-gradient-to-br");
      expect(button.className).toContain("from-heycat-orange");
      expect(button.className).toContain("to-heycat-orange-light");
    });

    it("renders secondary variant with orange border", () => {
      render(<Button variant="secondary">Secondary</Button>);
      const button = screen.getByRole("button", { name: "Secondary" });
      expect(button.className).toContain("border");
      expect(button.className).toContain("border-heycat-orange");
      expect(button.className).toContain("bg-white");
    });

    it("renders ghost variant with transparent background", () => {
      render(<Button variant="ghost">Ghost</Button>);
      const button = screen.getByRole("button", { name: "Ghost" });
      expect(button.className).toContain("bg-transparent");
    });

    it("renders danger variant with red background", () => {
      render(<Button variant="danger">Danger</Button>);
      const button = screen.getByRole("button", { name: "Danger" });
      expect(button.className).toContain("bg-error");
    });
  });

  describe("sizes", () => {
    it("renders small size", () => {
      render(<Button size="sm">Small</Button>);
      const button = screen.getByRole("button", { name: "Small" });
      expect(button.className).toContain("px-3");
      expect(button.className).toContain("py-1.5");
      expect(button.className).toContain("text-sm");
    });

    it("renders medium size by default", () => {
      render(<Button>Medium</Button>);
      const button = screen.getByRole("button", { name: "Medium" });
      expect(button.className).toContain("px-5");
      expect(button.className).toContain("py-2.5");
    });

    it("renders large size", () => {
      render(<Button size="lg">Large</Button>);
      const button = screen.getByRole("button", { name: "Large" });
      expect(button.className).toContain("px-6");
      expect(button.className).toContain("py-3");
      expect(button.className).toContain("text-lg");
    });
  });

  describe("states", () => {
    it("shows disabled state", () => {
      render(<Button disabled>Disabled</Button>);
      const button = screen.getByRole("button", { name: "Disabled" });
      expect(button.hasAttribute("disabled")).toBe(true);
      expect(button.className).toContain("disabled:opacity-50");
    });

    it("shows loading state with spinner", () => {
      render(<Button loading>Loading</Button>);
      const button = screen.getByRole("button", { name: /Loading/i });
      expect(button.hasAttribute("disabled")).toBe(true);
      // Check for spinner svg using testid
      const spinner = screen.getByTestId("button-spinner");
      expect(spinner).toBeDefined();
      expect(spinner.getAttribute("class")).toContain("animate-spin");
    });

    it("disables button when loading", () => {
      render(<Button loading>Submit</Button>);
      const button = screen.getByRole("button");
      expect(button.hasAttribute("disabled")).toBe(true);
    });
  });

  describe("interactions", () => {
    it("calls onClick when clicked", async () => {
      const user = userEvent.setup();
      const handleClick = vi.fn();
      render(<Button onClick={handleClick}>Click me</Button>);

      await user.click(screen.getByRole("button", { name: "Click me" }));
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it("does not call onClick when disabled", async () => {
      const user = userEvent.setup();
      const handleClick = vi.fn();
      render(
        <Button onClick={handleClick} disabled>
          Click me
        </Button>
      );

      await user.click(screen.getByRole("button", { name: "Click me" }));
      expect(handleClick).not.toHaveBeenCalled();
    });

    it("does not call onClick when loading", async () => {
      const user = userEvent.setup();
      const handleClick = vi.fn();
      render(
        <Button onClick={handleClick} loading>
          Click me
        </Button>
      );

      await user.click(screen.getByRole("button"));
      expect(handleClick).not.toHaveBeenCalled();
    });
  });

  describe("hover/press animations", () => {
    it("has hover elevation classes for primary variant", () => {
      render(<Button variant="primary">Primary</Button>);
      const button = screen.getByRole("button", { name: "Primary" });
      expect(button.className).toContain("hover:shadow-md");
      expect(button.className).toContain("hover:-translate-y-px");
    });

    it("has press reduction classes for primary variant", () => {
      render(<Button variant="primary">Primary</Button>);
      const button = screen.getByRole("button", { name: "Primary" });
      expect(button.className).toContain("active:translate-y-0");
      expect(button.className).toContain("active:shadow-sm");
    });
  });

  describe("asChild pattern", () => {
    it("renders as child element when asChild is true", () => {
      render(
        <Button asChild>
          <a href="/test">Link Button</a>
        </Button>
      );
      const link = screen.getByRole("link", { name: "Link Button" });
      expect(link).toBeDefined();
      expect(link.getAttribute("href")).toBe("/test");
    });
  });

  describe("custom className", () => {
    it("merges custom className with default styles", () => {
      render(<Button className="custom-class">Custom</Button>);
      const button = screen.getByRole("button", { name: "Custom" });
      expect(button.className).toContain("custom-class");
      expect(button.className).toContain("inline-flex");
    });
  });
});
