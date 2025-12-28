import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { Dashboard } from "./Dashboard";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock Tauri events
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock hooks
const mockStartRecording = vi.fn();
const mockDownloadModel = vi.fn();

vi.mock("../hooks/useRecording", () => ({
  useRecording: () => ({
    startRecording: mockStartRecording,
  }),
}));

vi.mock("../hooks/useMultiModelStatus", () => ({
  useMultiModelStatus: () => ({
    models: {
      isAvailable: false,
      downloadState: "idle",
      progress: 0,
      error: null,
    },
    downloadModel: mockDownloadModel,
  }),
}));

vi.mock("../hooks/useSettings", () => ({
  useSettings: () => ({
    settings: {
      audio: {
        selectedDevice: null,
      },
    },
  }),
}));

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Dashboard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("renders dashboard with all sections", async () => {
    render(<Dashboard />);

    // Page header
    expect(screen.getByRole("heading", { name: "Dashboard" })).toBeDefined();
    expect(
      screen.getByText("Welcome back! Here's your HeyCat status.")
    ).toBeDefined();

    // Status cards
    expect(screen.getByText("Recordings")).toBeDefined();
    expect(screen.getByText("Commands")).toBeDefined();

    // Quick action buttons
    expect(
      screen.getByRole("button", { name: "Start Recording" })
    ).toBeDefined();
    expect(screen.getByRole("button", { name: "Train Command" })).toBeDefined();
    expect(
      screen.getByRole("button", { name: "Download Model" })
    ).toBeDefined();

    // Recent activity section
    expect(screen.getByText("Recent Activity")).toBeDefined();

    // Wait for recordings to load
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("list_recordings");
    });
  });

  it("displays recording count from backend", async () => {
    mockInvoke.mockResolvedValue([
      {
        filename: "recording1.wav",
        file_path: "/path/to/recording1.wav",
        duration_secs: 30,
        created_at: "2025-01-15T12:00:00Z",
        file_size_bytes: 1024,
      },
      {
        filename: "recording2.wav",
        file_path: "/path/to/recording2.wav",
        duration_secs: 45,
        created_at: "2025-01-14T10:00:00Z",
        file_size_bytes: 2048,
      },
    ]);

    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText("2 recordings")).toBeDefined();
    });
  });

  it("shows empty state when no recordings exist", async () => {
    mockInvoke.mockResolvedValue([]);

    render(<Dashboard />);

    await waitFor(() => {
      expect(
        screen.getByText(/No recordings yet.*Click "Start Recording"/i)
      ).toBeDefined();
    });
  });

  it("shows recent recordings with play buttons and metadata", async () => {
    mockInvoke.mockResolvedValue([
      {
        filename: "test-recording.wav",
        file_path: "/path/to/test-recording.wav",
        duration_secs: 65, // 1:05
        created_at: "2025-01-15T12:00:00Z",
        file_size_bytes: 1024,
      },
    ]);

    render(<Dashboard />);

    await waitFor(() => {
      // Filename visible
      expect(screen.getByText("test-recording.wav")).toBeDefined();
      // Duration formatted
      expect(screen.getByText("1:05")).toBeDefined();
      // Play button
      expect(
        screen.getByRole("button", { name: /play test-recording.wav/i })
      ).toBeDefined();
    });
  });

  it("start recording button triggers recording", async () => {
    const user = userEvent.setup();
    render(<Dashboard />);

    await user.click(screen.getByRole("button", { name: "Start Recording" }));
    expect(mockStartRecording).toHaveBeenCalled();
  });

  it("download model button triggers download", async () => {
    const user = userEvent.setup();
    render(<Dashboard />);

    await user.click(screen.getByRole("button", { name: "Download Model" }));
    expect(mockDownloadModel).toHaveBeenCalledWith("tdt");
  });

  it("navigation cards trigger onNavigate callback", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    render(<Dashboard onNavigate={handleNavigate} />);

    // Click recordings card
    const recordingsCard = screen.getByText("Recordings").closest("[role=button]");
    await user.click(recordingsCard!);
    expect(handleNavigate).toHaveBeenCalledWith("recordings");

    // Click commands card
    const commandsCard = screen.getByText("Commands").closest("[role=button]");
    await user.click(commandsCard!);
    expect(handleNavigate).toHaveBeenCalledWith("commands");
  });

  it("view all link navigates to recordings page", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    mockInvoke.mockResolvedValue([
      {
        filename: "recording.wav",
        file_path: "/path/recording.wav",
        duration_secs: 30,
        created_at: "2025-01-15T12:00:00Z",
        file_size_bytes: 1024,
      },
    ]);

    render(<Dashboard onNavigate={handleNavigate} />);

    await waitFor(() => {
      expect(screen.getByText("View all")).toBeDefined();
    });

    await user.click(screen.getByText("View all"));
    expect(handleNavigate).toHaveBeenCalledWith("recordings");
  });
});
