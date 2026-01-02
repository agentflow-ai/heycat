/**
 * Tests for useRecordingsFilter hook.
 */

import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useRecordingsFilter } from "../useRecordingsFilter";
import type { RecordingInfo } from "../../pages/components/RecordingItem";

describe("useRecordingsFilter", () => {
  const recordings: RecordingInfo[] = [
    {
      filename: "meeting.wav",
      file_path: "/path/meeting.wav",
      duration_secs: 120,
      created_at: "2024-01-15T10:00:00Z",
      file_size_bytes: 1024,
      transcription: "Hello world",
      active_window_app_name: "Zoom",
    },
    {
      filename: "note.wav",
      file_path: "/path/note.wav",
      duration_secs: 60,
      created_at: "2024-01-16T10:00:00Z",
      file_size_bytes: 512,
    },
    {
      filename: "call.wav",
      file_path: "/path/call.wav",
      duration_secs: 180,
      created_at: "2024-01-14T10:00:00Z",
      file_size_bytes: 2048,
      transcription: "Important call",
    },
  ];

  it("returns all recordings with default filters", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    expect(result.current.filteredRecordings).toHaveLength(3);
    expect(result.current.hasActiveFilters).toBe(false);
  });

  it("filters by search query", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setSearchQuery("meeting");
    });

    expect(result.current.filteredRecordings).toHaveLength(1);
    expect(result.current.filteredRecordings[0].filename).toBe("meeting.wav");
    expect(result.current.hasActiveFilters).toBe(true);
  });

  it("filters by transcription status - transcribed", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setFilterOption("transcribed");
    });

    expect(result.current.filteredRecordings).toHaveLength(2);
    expect(
      result.current.filteredRecordings.every((r) => r.transcription)
    ).toBe(true);
  });

  it("filters by transcription status - pending", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setFilterOption("pending");
    });

    expect(result.current.filteredRecordings).toHaveLength(1);
    expect(result.current.filteredRecordings[0].filename).toBe("note.wav");
  });

  it("sorts by newest first (default)", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    expect(result.current.filteredRecordings[0].filename).toBe("note.wav");
    expect(result.current.filteredRecordings[2].filename).toBe("call.wav");
  });

  it("sorts by oldest first", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setSortOption("oldest");
    });

    expect(result.current.filteredRecordings[0].filename).toBe("call.wav");
    expect(result.current.filteredRecordings[2].filename).toBe("note.wav");
  });

  it("sorts by duration", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setSortOption("longest");
    });

    expect(result.current.filteredRecordings[0].duration_secs).toBe(180);

    act(() => {
      result.current.setSortOption("shortest");
    });

    expect(result.current.filteredRecordings[0].duration_secs).toBe(60);
  });

  it("clears all filters", () => {
    const { result } = renderHook(() =>
      useRecordingsFilter({ recordings })
    );

    act(() => {
      result.current.setSearchQuery("test");
      result.current.setFilterOption("transcribed");
      result.current.setSortOption("oldest");
    });

    expect(result.current.hasActiveFilters).toBe(true);

    act(() => {
      result.current.clearFilters();
    });

    expect(result.current.searchQuery).toBe("");
    expect(result.current.filterOption).toBe("all");
    expect(result.current.sortOption).toBe("newest");
    expect(result.current.hasActiveFilters).toBe(false);
  });
});
