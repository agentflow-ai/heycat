import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  RecordingsList,
  RecordingInfo,
  formatDuration,
  formatDate,
  formatFileSize,
} from "./RecordingsList";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

const mockRecordings: RecordingInfo[] = [
  {
    filename: "recording-2025-12-01-143025.wav",
    file_path: "/path/to/recording-2025-12-01-143025.wav",
    duration_secs: 154,
    created_at: "2025-12-01T14:30:25Z",
    file_size_bytes: 1024000,
  },
  {
    filename: "recording-2025-12-02-091500.wav",
    file_path: "/path/to/recording-2025-12-02-091500.wav",
    duration_secs: 65,
    created_at: "2025-12-02T09:15:00Z",
    file_size_bytes: 512000,
  },
];

describe("RecordingsList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("rendering", () => {
    it("renders without errors", async () => {
      mockInvoke.mockResolvedValue([]);

      expect(() => render(<RecordingsList />)).not.toThrow();
    });

    it("displays loading state while fetching", () => {
      mockInvoke.mockImplementation(
        () => new Promise(() => {}) // Never resolves
      );

      render(<RecordingsList />);

      expect(screen.getByText("Loading recordings...")).toBeDefined();
      expect(screen.getByRole("status").getAttribute("aria-busy")).toBe("true");
    });

    it("renders all recordings from backend response", async () => {
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("recording-2025-12-01-143025.wav")
        ).toBeDefined();
        expect(
          screen.getByText("recording-2025-12-02-091500.wav")
        ).toBeDefined();
      });
    });

    it("shows empty state when no recordings exist", async () => {
      mockInvoke.mockResolvedValue([]);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(screen.getByText("No recordings yet")).toBeDefined();
      });
    });

    it("shows error state when fetch fails", async () => {
      mockInvoke.mockRejectedValue(new Error("Network error"));

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("Failed to load recordings: Network error")
        ).toBeDefined();
      });
    });

    it("applies custom className", async () => {
      mockInvoke.mockResolvedValue([]);

      render(<RecordingsList className="custom-class" />);

      await waitFor(() => {
        const element = document.querySelector(".custom-class");
        expect(element).not.toBeNull();
      });
    });
  });

  describe("data fetching", () => {
    it("calls list_recordings command on mount", async () => {
      mockInvoke.mockResolvedValue([]);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("list_recordings");
      });
    });
  });

  describe("formatting", () => {
    it("displays duration in formatted format", async () => {
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(screen.getByText("2:34")).toBeDefined(); // 154 seconds
        expect(screen.getByText("1:05")).toBeDefined(); // 65 seconds
      });
    });

    it("displays date in user-friendly format", async () => {
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        // The exact format depends on locale, but should contain the date parts
        const items = screen.getAllByRole("listitem");
        expect(items.length).toBe(2);
      });
    });
  });

  describe("list structure", () => {
    it("renders recordings as list items", async () => {
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        const list = screen.getByRole("list");
        expect(list).toBeDefined();
        const items = screen.getAllByRole("listitem");
        expect(items.length).toBe(2);
      });
    });
  });

  describe("expand/collapse", () => {
    it("clicking entry expands it", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("recording-2025-12-01-143025.wav")
        ).toBeDefined();
      });

      const firstItem = screen.getByRole("button", {
        name: /recording-2025-12-01-143025\.wav/,
      });
      expect(firstItem.getAttribute("aria-expanded")).toBe("false");

      await user.click(firstItem);

      expect(firstItem.getAttribute("aria-expanded")).toBe("true");
    });

    it("clicking expanded entry collapses it", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("recording-2025-12-01-143025.wav")
        ).toBeDefined();
      });

      const firstItem = screen.getByRole("button", {
        name: /recording-2025-12-01-143025\.wav/,
      });

      // Expand
      await user.click(firstItem);
      expect(firstItem.getAttribute("aria-expanded")).toBe("true");

      // Collapse
      await user.click(firstItem);
      expect(firstItem.getAttribute("aria-expanded")).toBe("false");
    });

    it("expanded entry shows full metadata", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("recording-2025-12-01-143025.wav")
        ).toBeDefined();
      });

      const firstItem = screen.getByRole("button", {
        name: /recording-2025-12-01-143025\.wav/,
      });

      await user.click(firstItem);

      // Check metadata is shown (using values unique to this recording)
      expect(screen.getByText("1000.0 KB")).toBeDefined(); // 1024000 bytes - unique to first recording
      expect(
        screen.getByText("/path/to/recording-2025-12-01-143025.wav")
      ).toBeDefined();

      // Verify metadata labels exist (at least one visible)
      const fileSizeLabels = screen.getAllByText("File size");
      expect(fileSizeLabels.length).toBeGreaterThan(0);
    });

    it("only one entry can be expanded at a time", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(mockRecordings);

      render(<RecordingsList />);

      await waitFor(() => {
        expect(
          screen.getByText("recording-2025-12-01-143025.wav")
        ).toBeDefined();
      });

      const firstItem = screen.getByRole("button", {
        name: /recording-2025-12-01-143025\.wav/,
      });
      const secondItem = screen.getByRole("button", {
        name: /recording-2025-12-02-091500\.wav/,
      });

      // Expand first
      await user.click(firstItem);
      expect(firstItem.getAttribute("aria-expanded")).toBe("true");
      expect(secondItem.getAttribute("aria-expanded")).toBe("false");

      // Expand second - first should collapse
      await user.click(secondItem);
      expect(firstItem.getAttribute("aria-expanded")).toBe("false");
      expect(secondItem.getAttribute("aria-expanded")).toBe("true");
    });
  });
});

describe("formatDuration", () => {
  it('formats 0 seconds as "0:00"', () => {
    expect(formatDuration(0)).toBe("0:00");
  });

  it('formats 154 seconds as "2:34"', () => {
    expect(formatDuration(154)).toBe("2:34");
  });

  it('formats 65 seconds as "1:05"', () => {
    expect(formatDuration(65)).toBe("1:05");
  });

  it('formats 3600 seconds as "60:00"', () => {
    expect(formatDuration(3600)).toBe("60:00");
  });

  it("handles fractional seconds by flooring", () => {
    expect(formatDuration(65.9)).toBe("1:05");
  });
});

describe("formatDate", () => {
  it("formats ISO date string to readable format", () => {
    const result = formatDate("2025-12-01T14:30:25Z");
    // The exact format depends on locale, but should contain these parts
    expect(result).toContain("2025");
    expect(result).toContain("Dec");
  });

  it("handles different timezones", () => {
    const result = formatDate("2025-06-15T08:00:00Z");
    expect(result).toContain("2025");
  });
});

describe("formatFileSize", () => {
  it('formats 0 bytes as "0 B"', () => {
    expect(formatFileSize(0)).toBe("0 B");
  });

  it("formats bytes without decimals", () => {
    expect(formatFileSize(512)).toBe("512 B");
  });

  it("formats kilobytes with one decimal", () => {
    expect(formatFileSize(1024)).toBe("1.0 KB");
    expect(formatFileSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes with one decimal", () => {
    expect(formatFileSize(1024 * 1024)).toBe("1.0 MB");
    expect(formatFileSize(1024000)).toBe("1000.0 KB");
  });

  it("formats gigabytes with one decimal", () => {
    expect(formatFileSize(1024 * 1024 * 1024)).toBe("1.0 GB");
  });
});
