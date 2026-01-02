/**
 * Test utilities for ShortcutEditor component tests.
 */
import { vi } from "vitest";

/**
 * Create a mock event listener.
 */
export function createMockListener() {
  return vi.fn();
}

/**
 * Simulate a keyboard event.
 */
export function simulateKeyEvent(
  key: string,
  type: "keydown" | "keyup" = "keydown",
  options: Partial<KeyboardEventInit> = {}
): KeyboardEvent {
  return new KeyboardEvent(type, {
    key,
    bubbles: true,
    ...options,
  });
}

/**
 * Create mock Tauri invoke function.
 */
export function createMockInvoke() {
  return vi.fn();
}

/**
 * Create mock Tauri event listener.
 */
export function createMockEventListener() {
  const mockUnlisten = vi.fn();
  const mockListen = vi.fn().mockResolvedValue(mockUnlisten);
  return { mockListen, mockUnlisten };
}

/**
 * Default mock settings for tests.
 */
export const defaultMockSettings = {
  listening: { enabled: false, autoStartOnLaunch: false },
  audio: { selectedDevice: null },
  shortcuts: { distinguishLeftRight: false },
};

/**
 * Default props for ShortcutEditor component.
 */
export const defaultShortcutEditorProps = {
  open: true,
  onOpenChange: vi.fn(),
  shortcutName: "Toggle Recording",
  currentShortcut: "⌘⇧R",
  onSave: vi.fn() as (displayShortcut: string, backendShortcut: string) => void,
};

/**
 * Create a shortcut key event with modifier keys.
 */
export function createShortcutEvent(
  key: string,
  modifiers: {
    metaKey?: boolean;
    ctrlKey?: boolean;
    altKey?: boolean;
    shiftKey?: boolean;
  } = {}
): KeyboardEvent {
  return new KeyboardEvent("keydown", {
    key,
    bubbles: true,
    metaKey: modifiers.metaKey ?? false,
    ctrlKey: modifiers.ctrlKey ?? false,
    altKey: modifiers.altKey ?? false,
    shiftKey: modifiers.shiftKey ?? false,
  });
}
