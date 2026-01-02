/**
 * Tests for formatting utilities.
 *
 * These tests verify user-visible formatting behaviors:
 * - Duration formatting (MM:SS)
 * - Date formatting (localized)
 * - File size formatting (human-readable)
 * - Keyboard shortcut formatting
 */

import { describe, it, expect } from "vitest";
import {
  formatDuration,
  formatDurationLong,
  formatDate,
  formatFileSize,
  formatBackendKeyForDisplay,
  formatBackendKeyForBackend,
  isValidHotkey,
} from "../formatting";
import type { CapturedKeyEvent } from "../constants";

describe("duration formatting", () => {
  it("formats seconds as MM:SS", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(5)).toBe("0:05");
    expect(formatDuration(65)).toBe("1:05");
    expect(formatDuration(125)).toBe("2:05");
    expect(formatDuration(3600)).toBe("60:00");
  });

  it("formats duration as human-readable string", () => {
    expect(formatDurationLong(0)).toBe("0 sec");
    expect(formatDurationLong(5)).toBe("5 sec");
    expect(formatDurationLong(60)).toBe("1 min");
    expect(formatDurationLong(65)).toBe("1 min 5 sec");
    expect(formatDurationLong(125)).toBe("2 min 5 sec");
  });
});

describe("date formatting", () => {
  it("formats ISO date string to localized format", () => {
    const result = formatDate("2024-01-15T10:30:00Z");
    // Just verify it contains the key parts (locale-dependent)
    expect(result).toContain("2024");
    expect(result).toContain("15");
  });
});

describe("file size formatting", () => {
  it("formats bytes to human-readable sizes", () => {
    expect(formatFileSize(0)).toBe("0 B");
    expect(formatFileSize(100)).toBe("100 B");
    expect(formatFileSize(1024)).toBe("1.0 KB");
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(1048576)).toBe("1.0 MB");
    expect(formatFileSize(1073741824)).toBe("1.0 GB");
  });

  it("handles edge cases gracefully", () => {
    // Negative values
    expect(formatFileSize(-1)).toBe("0 B");
    expect(formatFileSize(-1000)).toBe("0 B");

    // Non-finite values
    expect(formatFileSize(Infinity)).toBe("0 B");
    expect(formatFileSize(-Infinity)).toBe("0 B");
    expect(formatFileSize(NaN)).toBe("0 B");
  });
});

describe("keyboard shortcut formatting", () => {
  // Helper to create a mock key event
  function createKeyEvent(overrides: Partial<CapturedKeyEvent>): CapturedKeyEvent {
    return {
      key_code: 0,
      key_name: "A",
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
      ...overrides,
    };
  }

  describe("formatBackendKeyForDisplay", () => {
    it("formats simple key", () => {
      const event = createKeyEvent({ key_name: "A" });
      expect(formatBackendKeyForDisplay(event)).toBe("A");
    });

    it("formats key with command modifier", () => {
      const event = createKeyEvent({ key_name: "A", command: true });
      expect(formatBackendKeyForDisplay(event)).toBe("⌘A");
    });

    it("formats key with multiple modifiers", () => {
      const event = createKeyEvent({
        key_name: "A",
        command: true,
        shift: true,
      });
      expect(formatBackendKeyForDisplay(event)).toBe("⌘⇧A");
    });

    it("formats fn key combinations", () => {
      const event = createKeyEvent({
        key_name: "F1",
        fn_key: true,
      });
      expect(formatBackendKeyForDisplay(event)).toBe("fnF1");
    });

    it("formats special keys with symbols", () => {
      expect(
        formatBackendKeyForDisplay(createKeyEvent({ key_name: "Up" }))
      ).toBe("↑");
      expect(
        formatBackendKeyForDisplay(createKeyEvent({ key_name: "Enter" }))
      ).toBe("↵");
      expect(
        formatBackendKeyForDisplay(createKeyEvent({ key_name: "Backspace" }))
      ).toBe("⌫");
    });

    it("formats media keys with emoji", () => {
      const event = createKeyEvent({
        key_name: "VolumeUp",
        is_media_key: true,
      });
      expect(formatBackendKeyForDisplay(event)).toBe("🔊");
    });

    it("distinguishes left/right modifiers when enabled", () => {
      const leftCmd = createKeyEvent({
        key_name: "A",
        command: true,
        command_left: true,
      });
      expect(formatBackendKeyForDisplay(leftCmd, true)).toBe("L⌘A");

      const rightCmd = createKeyEvent({
        key_name: "A",
        command: true,
        command_right: true,
      });
      expect(formatBackendKeyForDisplay(rightCmd, true)).toBe("R⌘A");
    });
  });

  describe("formatBackendKeyForBackend", () => {
    it("formats key with modifiers for backend", () => {
      const event = createKeyEvent({
        key_name: "A",
        command: true,
        shift: true,
      });
      expect(formatBackendKeyForBackend(event)).toBe("Command+Shift+A");
    });

    it("uses Function for fn key", () => {
      const event = createKeyEvent({
        key_name: "F1",
        fn_key: true,
      });
      expect(formatBackendKeyForBackend(event)).toBe("Function+F1");
    });
  });

  describe("isValidHotkey", () => {
    it("accepts regular key presses", () => {
      const event = createKeyEvent({ key_name: "A", pressed: true });
      expect(isValidHotkey(event)).toBe(true);
    });

    it("accepts modifier+key combinations", () => {
      const event = createKeyEvent({
        key_name: "A",
        command: true,
        pressed: true,
      });
      expect(isValidHotkey(event)).toBe(true);
    });

    it("rejects modifier-only key presses", () => {
      const event = createKeyEvent({
        key_name: "Command",
        command: true,
        pressed: true,
      });
      expect(isValidHotkey(event)).toBe(false);
    });

    it("accepts modifier-only key releases", () => {
      const event = createKeyEvent({
        key_name: "Command",
        command: false, // already released
        pressed: false,
      });
      expect(isValidHotkey(event)).toBe(true);
    });

    it("accepts media keys", () => {
      const event = createKeyEvent({
        key_name: "VolumeUp",
        is_media_key: true,
        pressed: true,
      });
      expect(isValidHotkey(event)).toBe(true);
    });
  });
});
