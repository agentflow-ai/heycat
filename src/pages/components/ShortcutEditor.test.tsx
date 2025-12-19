import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ShortcutEditor } from "./ShortcutEditor";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock Tauri listen
const mockUnlisten = vi.fn();
const mockListen = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("ShortcutEditor", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    shortcutName: "Toggle Recording",
    currentShortcut: "⌘⇧R",
    onSave: vi.fn() as (displayShortcut: string, backendShortcut: string) => void,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(mockUnlisten);
  });

  describe("Theming", () => {
    it("renders hotkey display with theme-aware styling", () => {
      render(<ShortcutEditor {...defaultProps} />);

      const kbd = screen.getByText("⌘⇧R");
      // Verify it uses theme-aware classes instead of hardcoded colors
      expect(kbd.className).toContain("bg-surface-elevated");
      expect(kbd.className).toContain("text-text-primary");
      // Ensure it does NOT have the broken hardcoded style
      expect(kbd.className).not.toContain("bg-neutral-100");
    });
  });

  describe("Recording Mode - Global Shortcut Management", () => {
    it("suspends global shortcut when entering recording mode", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("suspend_recording_shortcut");
      });
    });

    it("shows recording state after clicking Record button", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(screen.getByText("Press your shortcut...")).toBeDefined();
        expect(screen.getByRole("button", { name: "Recording..." })).toBeDefined();
      });
    });

    it("starts backend keyboard capture when entering recording mode", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        // Backend capture commands should be called
        expect(mockInvoke).toHaveBeenCalledWith("suspend_recording_shortcut");
        expect(mockInvoke).toHaveBeenCalledWith("start_shortcut_recording");
        // Should listen for key capture events
        expect(mockListen).toHaveBeenCalledWith("shortcut_key_captured", expect.any(Function));
      });
    });

    it("resumes global shortcut when Cancel is clicked while suspended", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(
        <ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />
      );

      // Enter recording mode (suspends shortcut)
      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("suspend_recording_shortcut");
      });

      // Clear mock to track resume call
      mockInvoke.mockClear();

      // Click Cancel
      await user.click(screen.getByRole("button", { name: "Cancel" }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("resume_recording_shortcut");
      });
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("does not call suspend if not entering recording mode", async () => {
      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      // Click cancel without entering recording mode
      await user.click(screen.getByRole("button", { name: "Cancel" }));

      // suspend should not have been called
      expect(mockInvoke).not.toHaveBeenCalledWith("suspend_recording_shortcut");
    });
  });

  describe("Modal Behavior", () => {
    it("shows correct shortcut name in modal header", () => {
      render(<ShortcutEditor {...defaultProps} shortcutName="Custom Action" />);

      expect(screen.getByText(/Set a new shortcut for "Custom Action"/)).toBeDefined();
    });

    it("displays current shortcut initially", () => {
      render(<ShortcutEditor {...defaultProps} currentShortcut="⌘K" />);

      expect(screen.getByText("⌘K")).toBeDefined();
    });

    it("does not render when open is false", () => {
      render(<ShortcutEditor {...defaultProps} open={false} />);

      expect(screen.queryByText("Change Keyboard Shortcut")).toBeNull();
    });

    it("calls onOpenChange when Cancel is clicked", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(<ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />);

      await user.click(screen.getByRole("button", { name: "Cancel" }));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("calls onOpenChange when close button is clicked", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(<ShortcutEditor {...defaultProps} onOpenChange={onOpenChange} />);

      await user.click(screen.getByRole("button", { name: "Close" }));

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("disables Save button when no changes have been made", () => {
      render(<ShortcutEditor {...defaultProps} />);

      const saveButton = screen.getByRole("button", { name: "Save" });
      expect(saveButton.hasAttribute("disabled")).toBe(true);
    });
  });

  describe("Backend Key Capture Integration", () => {
    it("records shortcut when backend emits non-modifier key event", async () => {
      // Capture the event callback when listen is called
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      const onSave = vi.fn();
      render(<ShortcutEditor {...defaultProps} onSave={onSave} />);

      // Enter recording mode
      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate backend emitting a key event with fn modifier
      capturedCallback?.({
        payload: {
          key_code: 0x04, // A key
          key_name: "A",
          fn_key: true,
          command: true,
          control: false,
          alt: false,
          shift: false,
          pressed: true,
        },
      });

      // Should show the recorded shortcut
      await waitFor(() => {
        expect(screen.getByText("fn⌘A")).toBeDefined();
      });

      // Save button should be enabled now
      const saveButton = screen.getByRole("button", { name: "Save" });
      expect(saveButton.hasAttribute("disabled")).toBe(false);
    });

    it("ignores key release events", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate backend emitting a key release event
      capturedCallback?.({
        payload: {
          key_code: 0x04,
          key_name: "A",
          fn_key: false,
          command: false,
          control: false,
          alt: false,
          shift: false,
          pressed: false, // Release event
        },
      });

      // Should still be recording
      expect(screen.getByText("Press your shortcut...")).toBeDefined();
    });

    it("accepts modifier-only key events as valid hotkeys", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate backend emitting a modifier-only event (Command key alone)
      capturedCallback?.({
        payload: {
          key_code: 0xE3,
          key_name: "Command",
          fn_key: false,
          command: true,
          command_left: true,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: false,
          shift_left: false,
          shift_right: false,
          pressed: true,
          is_media_key: false,
        },
      });

      // Should record modifier-only as valid hotkey
      await waitFor(() => {
        expect(screen.getByText("⌘")).toBeDefined();
      });
    });

    it("displays media keys with symbols", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate backend emitting a Play/Pause media key event
      capturedCallback?.({
        payload: {
          key_code: 0x34,
          key_name: "PlayPause",
          fn_key: false,
          command: false,
          command_left: false,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: false,
          shift_left: false,
          shift_right: false,
          pressed: true,
          is_media_key: true,
        },
      });

      // Should display media key symbol
      await waitFor(() => {
        expect(screen.getByText("⏯")).toBeDefined();
      });
    });

    it("displays Volume Up media key", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate backend emitting Volume Up media key event
      capturedCallback?.({
        payload: {
          key_code: 0x48,
          key_name: "VolumeUp",
          fn_key: false,
          command: false,
          command_left: false,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: false,
          shift_left: false,
          shift_right: false,
          pressed: true,
          is_media_key: true,
        },
      });

      // Should display Volume Up symbol
      await waitFor(() => {
        expect(screen.getByText("🔊")).toBeDefined();
      });
    });

    it("displays fn key modifier", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate fn key alone pressed
      capturedCallback?.({
        payload: {
          key_code: 0x3F,
          key_name: "fn",
          fn_key: true,
          command: false,
          command_left: false,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: false,
          shift_left: false,
          shift_right: false,
          pressed: true,
          is_media_key: false,
        },
      });

      // Should display fn
      await waitFor(() => {
        expect(screen.getByText("fn")).toBeDefined();
      });
    });

    it("displays Space key correctly", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate Space key pressed
      capturedCallback?.({
        payload: {
          key_code: 0x31,
          key_name: "Space",
          fn_key: false,
          command: false,
          command_left: false,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: false,
          shift_left: false,
          shift_right: false,
          pressed: true,
          is_media_key: false,
        },
      });

      // Should display "Space"
      await waitFor(() => {
        expect(screen.getByText("Space")).toBeDefined();
      });
    });

    it("displays complex shortcut with multiple modifiers", async () => {
      let capturedCallback: ((event: { payload: unknown }) => void) | undefined;
      mockListen.mockImplementation((_eventName: string, callback: (event: { payload: unknown }) => void) => {
        capturedCallback = callback;
        return Promise.resolve(mockUnlisten);
      });

      const user = userEvent.setup();
      render(<ShortcutEditor {...defaultProps} />);

      await user.click(screen.getByRole("button", { name: "Record New Shortcut" }));

      await waitFor(() => {
        expect(capturedCallback).toBeDefined();
      });

      // Simulate Cmd+Shift+A
      capturedCallback?.({
        payload: {
          key_code: 0x00,
          key_name: "A",
          fn_key: false,
          command: true,
          command_left: true,
          command_right: false,
          control: false,
          control_left: false,
          control_right: false,
          alt: false,
          alt_left: false,
          alt_right: false,
          shift: true,
          shift_left: true,
          shift_right: false,
          pressed: true,
          is_media_key: false,
        },
      });

      // Should display "⌘⇧A"
      await waitFor(() => {
        expect(screen.getByText("⌘⇧A")).toBeDefined();
      });
    });
  });
});
