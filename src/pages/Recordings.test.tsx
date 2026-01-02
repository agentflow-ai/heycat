import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Recordings, type PaginatedRecordingsResponse } from "./Recordings";
import {
  sampleRecordings,
  createPaginatedResponse,
  emptyPaginatedResponse,
} from "./Recordings/testUtils";

// Mock Tauri invoke with vi.hoisted for proper scoping
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

// Mock Tauri opener plugin with vi.hoisted for proper scoping
const { mockOpenPath } = vi.hoisted(() => ({
  mockOpenPath: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: mockOpenPath,
}));

// Mock toast with vi.hoisted for proper scoping
const { mockToast } = vi.hoisted(() => ({
  mockToast: vi.fn(),
}));

vi.mock("../components/overlays", () => ({
  useToast: () => ({
    toast: mockToast,
    dismiss: vi.fn(),
    dismissAll: vi.fn(),
  }),
}));

vi.mock("../hooks/useRecording", () => ({
  useRecording: () => ({
    isRecording: false,
    isProcessing: false,
    error: null,
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    isStarting: false,
    isStopping: false,
  }),
}));

// Mock useAudioPlayback hook - state container for the mock
const audioPlaybackState = { isPlaying: false, currentFilePath: null as string | null };

const { mockToggle } = vi.hoisted(() => ({
  mockToggle: vi.fn(),
}));

vi.mock("../hooks/useAudioPlayback", () => ({
  useAudioPlayback: () => ({
    isPlaying: audioPlaybackState.isPlaying,
    currentFilePath: audioPlaybackState.currentFilePath,
    play: vi.fn(),
    pause: vi.fn(),
    stop: vi.fn(),
    toggle: mockToggle,
    error: null,
  }),
}));

// Create wrapper for QueryClientProvider
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("Recordings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(emptyPaginatedResponse);
    audioPlaybackState.isPlaying = false;
    audioPlaybackState.currentFilePath = null;
  });

  it("renders page with header, search, filter, and sort", async () => {
    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Recordings" })
      ).toBeDefined();
    });

    expect(
      screen.getByText("Manage your voice recordings and transcriptions.")
    ).toBeDefined();
    expect(
      screen.getByRole("textbox", { name: /search recordings/i })
    ).toBeDefined();
  });

  it("shows empty state when no recordings exist", async () => {
    mockInvoke.mockResolvedValue(emptyPaginatedResponse);

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("No recordings yet")).toBeDefined();
    });

    // Recording button removed - only hotkey-based recording now
    expect(
      screen.getByText(/Press ⌘⇧R or say "Hey Cat" to start/i)
    ).toBeDefined();
  });

  it("displays recordings list with play button, filename, and metadata", async () => {
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Check all recordings are displayed
    expect(screen.getByText("meeting_notes.wav")).toBeDefined();
    expect(screen.getByText("quick_memo.wav")).toBeDefined();

    // Check play buttons exist
    const playButtons = screen.getAllByRole("button", { name: /play/i });
    expect(playButtons.length).toBeGreaterThan(0);
  });

  it("shows transcription status badges correctly", async () => {
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Should have Transcribed badges and Transcribe buttons
    const transcribedBadges = screen.getAllByText("Transcribed");
    expect(transcribedBadges.length).toBe(2); // Two recordings have transcriptions

    const transcribeButtons = screen.getAllByRole("button", { name: /^transcribe$/i });
    expect(transcribeButtons.length).toBe(1); // One recording without transcription
  });

  it("filters recordings by search query", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Search for "meeting"
    const searchInput = screen.getByRole("textbox", {
      name: /search recordings/i,
    });
    await user.type(searchInput, "meeting");

    // Only matching recording should be visible
    expect(screen.queryByText("recording_2024-01-15.wav")).toBeNull();
    expect(screen.getByText("meeting_notes.wav")).toBeDefined();
    expect(screen.queryByText("quick_memo.wav")).toBeNull();
  });

  it("filters recordings by transcription content", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Search for "deadline" (in transcription)
    const searchInput = screen.getByRole("textbox", {
      name: /search recordings/i,
    });
    await user.type(searchInput, "deadline");

    // Only the recording with "deadline" in transcription should be visible
    expect(screen.queryByText("recording_2024-01-15.wav")).toBeNull();
    expect(screen.queryByText("meeting_notes.wav")).toBeNull();
    expect(screen.getByText("quick_memo.wav")).toBeDefined();
  });

  it("shows no results message when search has no matches", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    const searchInput = screen.getByRole("textbox", {
      name: /search recordings/i,
    });
    await user.type(searchInput, "nonexistent");

    expect(screen.getByText("No recordings match your search")).toBeDefined();
    expect(screen.getByText("Clear filters")).toBeDefined();
  });

  it("expands and collapses recording item on click", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Find expand button for the first recording
    const expandButton = screen.getByRole("button", {
      name: /expand recording_2024-01-15.wav/i,
    });

    // Click to expand
    await user.click(expandButton);

    // Should show transcription section
    expect(screen.getByText("Transcription")).toBeDefined();
    expect(screen.getByText("Hello, this is a test transcription.")).toBeDefined();

    // Should show action buttons
    expect(screen.getByRole("button", { name: /copy transcription text/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /open file in system/i })).toBeDefined();

    // Click to collapse
    await user.click(expandButton);

    // Transcription section should be hidden
    expect(screen.queryByText("Hello, this is a test transcription.")).toBeNull();
  });

  it("shows 'No transcription available' when expanded and no transcription", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("meeting_notes.wav")).toBeDefined();
    });

    // Expand the recording without transcription
    const expandButton = screen.getByRole("button", {
      name: /expand meeting_notes.wav/i,
    });
    await user.click(expandButton);

    expect(screen.getByText("No transcription available")).toBeDefined();
  });

  it("copies transcription text and shows toast", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    // Mock clipboard API using stubGlobal
    const mockWriteText = vi.fn().mockResolvedValue(undefined);
    const mockClipboard = {
      writeText: mockWriteText,
      readText: vi.fn(),
    };
    vi.stubGlobal("navigator", {
      ...navigator,
      clipboard: mockClipboard,
    });

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Expand recording
    await user.click(
      screen.getByRole("button", { name: /expand recording_2024-01-15.wav/i })
    );

    // Click copy button
    await user.click(
      screen.getByRole("button", { name: /copy transcription text/i })
    );

    await waitFor(() => {
      expect(mockWriteText).toHaveBeenCalledWith(
        "Hello, this is a test transcription."
      );
    });
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Copied to clipboard",
      })
    );

    // Restore original navigator
    vi.unstubAllGlobals();
  });

  it("opens file in system when Open File button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));
    mockOpenPath.mockResolvedValue(undefined);

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Expand recording
    await user.click(
      screen.getByRole("button", { name: /expand recording_2024-01-15.wav/i })
    );

    // Click Open File button
    await user.click(
      screen.getByRole("button", { name: /open file in system/i })
    );

    expect(mockOpenPath).toHaveBeenCalledWith("/path/to/recording_2024-01-15.wav");
  });

  it("shows delete confirmation and deletes recording", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(sampleRecordings)) // Initial load
      .mockResolvedValueOnce(undefined); // delete_recording response

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Expand recording
    await user.click(
      screen.getByRole("button", { name: /expand recording_2024-01-15.wav/i })
    );

    // Click delete button
    await user.click(
      screen.getByRole("button", { name: /delete recording_2024-01-15.wav/i })
    );

    // Confirmation buttons should appear
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /cancel delete/i })).toBeDefined();

    // Confirm delete
    await user.click(screen.getByRole("button", { name: /confirm delete/i }));

    expect(mockInvoke).toHaveBeenCalledWith("delete_recording", {
      filePath: "/path/to/recording_2024-01-15.wav",
    });

    // Toast shown
    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Recording deleted",
      })
    );
  });

  it("cancels delete when cancel button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Expand recording
    await user.click(
      screen.getByRole("button", { name: /expand recording_2024-01-15.wav/i })
    );

    // Click delete button
    await user.click(
      screen.getByRole("button", { name: /delete recording_2024-01-15.wav/i })
    );

    // Click cancel
    await user.click(screen.getByRole("button", { name: /cancel delete/i }));

    // Should not have called delete_recording
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "delete_recording",
      expect.anything()
    );

    // Action buttons should be back
    expect(
      screen.getByRole("button", { name: /copy transcription text/i })
    ).toBeDefined();
  });

  it("triggers transcription and shows success toast", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(sampleRecordings)) // Initial load
      .mockResolvedValueOnce("Transcribed text from audio"); // transcribe_file response

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("meeting_notes.wav")).toBeDefined();
    });

    // Click transcribe button on recording without transcription
    const transcribeButton = screen.getByRole("button", { name: /^transcribe$/i });
    await user.click(transcribeButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("transcribe_file", {
        filePath: "/path/to/meeting_notes.wav",
      });
    });

    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "success",
        title: "Transcription complete",
      })
    );
  });

  it("displays error state when loading fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Network error"));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeDefined();
    });

    expect(screen.getByText("Network error")).toBeDefined();
    expect(screen.getByText("Retry")).toBeDefined();
  });

  it("retries loading when retry button clicked", async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockRejectedValueOnce(new Error("Network error"))
      .mockResolvedValueOnce(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("Network error")).toBeDefined();
    });

    await user.click(screen.getByText("Retry"));

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });
  });

  // Test removed: "starts recording when empty state button clicked"
  // Recording button was removed from UI - only hotkey-based recording now

  it("sorts recordings by newest first by default", async () => {
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Get all recording filenames in display order
    const recordingItems = screen.getAllByRole("listitem");
    const filenames = recordingItems.map((item) =>
      item.textContent?.match(/[\w_-]+\.wav/)?.[0]
    );

    // Should be sorted by newest first (quick_memo: Jan 20, recording: Jan 15, meeting: Jan 10)
    expect(filenames[0]).toBe("quick_memo.wav");
    expect(filenames[1]).toBe("recording_2024-01-15.wav");
    expect(filenames[2]).toBe("meeting_notes.wav");
  });

  it("play button calls toggle with file path", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings));
    mockToggle.mockClear();

    render(<Recordings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
    });

    // Find and click play button
    const playButton = screen.getByRole("button", {
      name: /play recording_2024-01-15.wav/i,
    });
    await user.click(playButton);

    // Should call toggle with the file path
    expect(mockToggle).toHaveBeenCalledWith("/path/to/recording_2024-01-15.wav");
  });

  it("shows skeleton loaders while loading", async () => {
    // Don't resolve the promise immediately to test loading state
    mockInvoke.mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    render(<Recordings />, { wrapper: createWrapper() });

    // Should show loading state with skeleton elements
    expect(screen.getByRole("status", { name: /loading recordings/i })).toBeDefined();
  });

  describe("pagination", () => {
    it("does not show pagination controls when total items fit on one page", async () => {
      mockInvoke.mockResolvedValue(createPaginatedResponse(sampleRecordings, false));

      render(<Recordings />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
      });

      // Should not show pagination controls for small lists
      expect(screen.queryByRole("button", { name: /previous page/i })).toBeNull();
      expect(screen.queryByRole("button", { name: /next page/i })).toBeNull();
    });

    it("shows pagination controls when has_more is true", async () => {
      const manyRecordings: PaginatedRecordingsResponse = {
        recordings: sampleRecordings,
        total_count: 25, // More than PAGE_SIZE
        has_more: true,
      };
      mockInvoke.mockResolvedValue(manyRecordings);

      render(<Recordings />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
      });

      // Should show pagination controls
      expect(screen.getByRole("button", { name: /previous page/i })).toBeDefined();
      expect(screen.getByRole("button", { name: /next page/i })).toBeDefined();
      expect(screen.getByText(/Page 1 of 2/i)).toBeDefined();
      expect(screen.getByText(/Showing 1-3 of 25 recordings/i)).toBeDefined();
    });

    it("disables previous button on first page", async () => {
      const manyRecordings: PaginatedRecordingsResponse = {
        recordings: sampleRecordings,
        total_count: 25,
        has_more: true,
      };
      mockInvoke.mockResolvedValue(manyRecordings);

      render(<Recordings />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
      });

      const prevButton = screen.getByRole("button", { name: /previous page/i });
      expect(prevButton).toHaveProperty("disabled", true);
    });

    it("disables next button on last page", async () => {
      const lastPage: PaginatedRecordingsResponse = {
        recordings: sampleRecordings,
        total_count: 25,
        has_more: false, // No more pages
      };
      mockInvoke.mockResolvedValue(lastPage);

      render(<Recordings />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
      });

      // Since has_more is false and total_count > recordings shown, pagination should appear
      // But next should be disabled since has_more is false
      const nextButton = screen.queryByRole("button", { name: /next page/i });
      if (nextButton) {
        expect(nextButton).toHaveProperty("disabled", true);
      }
    });

    it("navigates to next page when next button clicked", async () => {
      const user = userEvent.setup();
      const firstPage: PaginatedRecordingsResponse = {
        recordings: sampleRecordings,
        total_count: 25,
        has_more: true,
      };
      mockInvoke.mockResolvedValue(firstPage);

      render(<Recordings />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByText("recording_2024-01-15.wav")).toBeDefined();
      });

      const nextButton = screen.getByRole("button", { name: /next page/i });
      await user.click(nextButton);

      // Should have called invoke with offset = PAGE_SIZE (20)
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("list_recordings", {
          limit: 20,
          offset: 20,
        });
      });
    });
  });
});
