/**
 * Toast notification tests
 * Tests focus on user-visible behavior per TESTING.md guidelines
 */
import { render, screen, act, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { ToastProvider, useToast } from "../toast";
import type { ToastOptions } from "../toast";

// Helper component to trigger toasts
function ToastTrigger({ options }: { options: ToastOptions }) {
  const { toast, dismissAll, toasts } = useToast();
  return (
    <div>
      <button onClick={() => toast(options)}>Show Toast</button>
      <button onClick={dismissAll}>Dismiss All</button>
      <span data-testid="toast-count">{toasts.length}</span>
    </div>
  );
}

function renderWithProvider(options: ToastOptions) {
  return render(
    <ToastProvider>
      <ToastTrigger options={options} />
    </ToastProvider>
  );
}

describe("Toast Notifications", () => {
  it("shows toast with title and description when triggered", async () => {
    const user = userEvent.setup();
    renderWithProvider({
      type: "success",
      title: "Transcription complete",
      description: "Hello, this is a test",
    });

    await user.click(screen.getByText("Show Toast"));

    expect(screen.getByText("Transcription complete")).toBeInTheDocument();
    expect(screen.getByText("Hello, this is a test")).toBeInTheDocument();
    expect(screen.getByTestId("toast")).toHaveAttribute("data-toast-type", "success");
  });

  it("close button dismisses toast immediately", async () => {
    const user = userEvent.setup();
    renderWithProvider({
      type: "info",
      title: "Dismissable toast",
      duration: null, // Prevent auto-dismiss for this test
    });

    await user.click(screen.getByText("Show Toast"));
    expect(screen.getByText("Dismissable toast")).toBeInTheDocument();

    await user.click(screen.getByLabelText("Dismiss notification"));

    // Toast triggers exit animation, then removes from DOM after 200ms
    await waitFor(
      () => {
        expect(screen.queryByText("Dismissable toast")).not.toBeInTheDocument();
      },
      { timeout: 500 }
    );
  });

  it("action button works and triggers callback", async () => {
    const user = userEvent.setup();
    const actionCallback = vi.fn();

    renderWithProvider({
      type: "success",
      title: "With action",
      duration: null,
      action: {
        label: "Copy to Clipboard",
        onClick: actionCallback,
      },
    });

    await user.click(screen.getByText("Show Toast"));
    await user.click(screen.getByText("Copy to Clipboard"));

    expect(actionCallback).toHaveBeenCalledTimes(1);
  });

  it("multiple toasts stack correctly (max 3 visible)", async () => {
    const user = userEvent.setup();

    // Custom component to show multiple toasts
    function MultiToastTrigger() {
      const { toast, toasts } = useToast();
      return (
        <div>
          <button
            onClick={() =>
              toast({ type: "info", title: `Toast ${toasts.length + 1}`, duration: null })
            }
          >
            Add Toast
          </button>
          <span data-testid="toast-count">{toasts.length}</span>
        </div>
      );
    }

    render(
      <ToastProvider>
        <MultiToastTrigger />
      </ToastProvider>
    );

    // Add 4 toasts
    for (let i = 0; i < 4; i++) {
      await user.click(screen.getByText("Add Toast"));
    }

    // Only 3 should be visible in the container
    const toastElements = screen.getAllByTestId("toast");
    expect(toastElements).toHaveLength(3);

    // But state tracks all 4
    expect(screen.getByTestId("toast-count")).toHaveTextContent("4");
  });

  it("dismissAll removes all toasts", async () => {
    const user = userEvent.setup();
    renderWithProvider({ type: "info", title: "Test toast", duration: null });

    // Add a toast
    await user.click(screen.getByText("Show Toast"));
    expect(screen.getByTestId("toast-count")).toHaveTextContent("1");

    // Dismiss all
    await user.click(screen.getByText("Dismiss All"));
    expect(screen.getByTestId("toast-count")).toHaveTextContent("0");
  });

  it("displays correct toast type styling", async () => {
    const user = userEvent.setup();

    // Test each type shows correct data attribute
    const types: Array<"success" | "error" | "warning" | "info"> = [
      "success",
      "error",
      "warning",
      "info",
    ];

    for (const type of types) {
      const { unmount } = renderWithProvider({ type, title: `${type} toast`, duration: null });
      await user.click(screen.getByText("Show Toast"));
      expect(screen.getByTestId("toast")).toHaveAttribute("data-toast-type", type);
      unmount();
    }
  });

  it("useToast throws error when used outside provider", () => {
    // Suppress console.error for this test
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    function BadComponent() {
      useToast();
      return null;
    }

    expect(() => render(<BadComponent />)).toThrow(
      "useToast must be used within a ToastProvider"
    );

    consoleSpy.mockRestore();
  });

  it("error toast has aria-live assertive for screen readers", async () => {
    const user = userEvent.setup();
    renderWithProvider({
      type: "error",
      title: "Critical error",
      duration: null,
    });

    await user.click(screen.getByText("Show Toast"));

    const toast = screen.getByTestId("toast");
    expect(toast).toHaveAttribute("aria-live", "assertive");
  });

  it("non-error toasts have aria-live polite", async () => {
    const user = userEvent.setup();
    renderWithProvider({
      type: "success",
      title: "Success",
      duration: null,
    });

    await user.click(screen.getByText("Show Toast"));

    const toast = screen.getByTestId("toast");
    expect(toast).toHaveAttribute("aria-live", "polite");
  });
});
