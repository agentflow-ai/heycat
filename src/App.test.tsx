/* v8 ignore file -- @preserve */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import * as useCatOverlayModule from "./hooks/useCatOverlay";
import * as useAppStatusModule from "./hooks/useAppStatus";
import * as useRecordingModule from "./hooks/useRecording";
import * as useMultiModelStatusModule from "./hooks/useMultiModelStatus";
import * as useSettingsModule from "./hooks/useSettings";

vi.mock("./hooks/useCatOverlay");
vi.mock("./hooks/useAppStatus");
vi.mock("./hooks/useRecording");
vi.mock("./hooks/useMultiModelStatus");
vi.mock("./hooks/useSettings", () => ({
  useSettings: vi.fn(),
  initializeSettings: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockUseCatOverlay = vi.mocked(useCatOverlayModule.useCatOverlay);
const mockUseAppStatus = vi.mocked(useAppStatusModule.useAppStatus);
const mockUseRecording = vi.mocked(useRecordingModule.useRecording);
const mockUseMultiModelStatus = vi.mocked(useMultiModelStatusModule.useMultiModelStatus);
const mockUseSettings = vi.mocked(useSettingsModule.useSettings);

describe("App Integration", () => {
  const defaultAppStatusMock: useAppStatusModule.UseAppStatusResult = {
    status: "idle",
    isRecording: false,
    isTranscribing: false,
    error: null,
  };

  const defaultRecordingMock: useRecordingModule.UseRecordingResult = {
    isRecording: false,
    isProcessing: false,
    error: null,
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    isStarting: false,
    isStopping: false,
  };

  const defaultMultiModelStatusMock: useMultiModelStatusModule.UseMultiModelStatusResult = {
    models: {
      isAvailable: true,
      downloadState: "idle" as const,
      progress: 0,
      error: null,
    },
    downloadModel: vi.fn(),
    refreshStatus: vi.fn(),
  };

  const defaultCatOverlayMock = {
    isRecording: false,
    overlayMode: "hidden" as useCatOverlayModule.OverlayMode,
  };

  const defaultSettingsMock: useSettingsModule.UseSettingsReturn = {
    settings: {
      audio: { selectedDevice: null },
      shortcuts: { distinguishLeftRight: false, recordingMode: "toggle" },
    },
    isLoading: false,
    updateAudioDevice: vi.fn(),
    updateDistinguishLeftRight: vi.fn(),
    updateRecordingMode: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseCatOverlay.mockReturnValue(defaultCatOverlayMock);
    mockUseAppStatus.mockReturnValue(defaultAppStatusMock);
    mockUseRecording.mockReturnValue(defaultRecordingMock);
    mockUseMultiModelStatus.mockReturnValue(defaultMultiModelStatusMock);
    mockUseSettings.mockReturnValue(defaultSettingsMock);
  });

  it("renders AppShell with navigation", async () => {
    render(<App />);

    await waitFor(() => {
      // Check for navigation element
      expect(screen.getByRole("navigation", { name: "Main navigation" })).toBeDefined();
    });
  });

  it("renders the header with HeyCat branding", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("HeyCat")).toBeDefined();
    });
  });

  it("shows status pill when recording", async () => {
    mockUseAppStatus.mockReturnValue({
      ...defaultAppStatusMock,
      status: "recording",
      isRecording: true,
    });

    render(<App />);

    await waitFor(() => {
      // Status pill should show Recording state
      expect(screen.getByRole("status")).toBeDefined();
      expect(screen.getByText("Recording")).toBeDefined();
    });
  });
});
