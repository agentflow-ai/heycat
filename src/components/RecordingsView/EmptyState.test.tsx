import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  describe("when no recordings exist (hasFiltersActive=false)", () => {
    it('displays "No recordings yet" message', () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(screen.getByText("No recordings yet")).toBeDefined();
    });

    it("displays helpful description to make first recording", () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(
        screen.getByText("Make your first recording to see it here")
      ).toBeDefined();
    });

    it("does not show clear filters button", () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(screen.queryByRole("button")).toBeNull();
    });
  });

  describe("when filters match no recordings (hasFiltersActive=true)", () => {
    it('displays "No recordings match your filters" message', () => {
      render(<EmptyState hasFiltersActive={true} />);

      expect(screen.getByText("No recordings match your filters")).toBeDefined();
    });

    it("shows clear filters button when onClearFilters is provided", () => {
      const onClearFilters = vi.fn();
      render(
        <EmptyState hasFiltersActive={true} onClearFilters={onClearFilters} />
      );

      expect(screen.getByRole("button", { name: "Clear filters" })).toBeDefined();
    });

    it("calls onClearFilters when clear button is clicked", () => {
      const onClearFilters = vi.fn();
      render(
        <EmptyState hasFiltersActive={true} onClearFilters={onClearFilters} />
      );

      fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));

      expect(onClearFilters).toHaveBeenCalledTimes(1);
    });
  });
});
