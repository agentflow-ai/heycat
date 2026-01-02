/**
 * Test utilities for Recordings component tests.
 */
import { vi } from "vitest";
import type { RecordingInfo, PaginatedRecordingsResponse } from "../components/RecordingItem";

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
  type: "keydown" | "keyup" = "keydown"
): KeyboardEvent {
  return new KeyboardEvent(type, { key, bubbles: true });
}

/**
 * Sample recording data for tests.
 */
export const sampleRecordings: RecordingInfo[] = [
  {
    filename: "recording_2024-01-15.wav",
    file_path: "/path/to/recording_2024-01-15.wav",
    duration_secs: 120,
    created_at: "2024-01-15T14:30:00Z",
    file_size_bytes: 3600000,
    transcription: "Hello, this is a test transcription.",
  },
  {
    filename: "meeting_notes.wav",
    file_path: "/path/to/meeting_notes.wav",
    duration_secs: 300,
    created_at: "2024-01-10T10:00:00Z",
    file_size_bytes: 9000000,
  },
  {
    filename: "quick_memo.wav",
    file_path: "/path/to/quick_memo.wav",
    duration_secs: 30,
    created_at: "2024-01-20T08:15:00Z",
    file_size_bytes: 900000,
    transcription: "Quick memo about the project deadline.",
  },
];

/**
 * Create a paginated response for recordings.
 */
export function createPaginatedResponse(
  recordings: RecordingInfo[],
  hasMore = false
): PaginatedRecordingsResponse {
  return {
    recordings,
    total_count: recordings.length,
    has_more: hasMore,
  };
}

/**
 * Empty paginated response.
 */
export const emptyPaginatedResponse: PaginatedRecordingsResponse = {
  recordings: [],
  total_count: 0,
  has_more: false,
};

/**
 * Create mock audio playback state.
 */
export function createAudioPlaybackState() {
  return { isPlaying: false, currentFilePath: null as string | null };
}
