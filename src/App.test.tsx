/* v8 ignore file -- @preserve */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import * as useCatOverlayModule from "./hooks/useCatOverlay";
import * as useAppStatusModule from "./hooks/useAppStatus";
import * as useAutoStartListeningModule from "./hooks/useAutoStartListening";
import * as useListeningModule from "./hooks/useListening";
import * as useRecordingModule from "./hooks/useRecording";
import * as useMultiModelStatusModule from "./hooks/useMultiModelStatus";
import * as useSettingsModule from "./hooks/useSettings";

vi.mock("./hooks/useCatOverlay");
vi.mock("./hooks/useAppStatus");
vi.mock("./hooks/useAutoStartListening");
vi.mock("./hooks/useListening");
vi.mock("./hooks/useRecording");
vi.mock("./hooks/useMultiModelStatus");
vi.mock("./hooks/useSettings");
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockUseCatOverlay = vi.mocked(useCatOverlayModule.useCatOverlay);
const mockUseAppStatus = vi.mocked(useAppStatusModule.useAppStatus);
const mockUseAutoStartListening = vi.mocked(useAutoStartListeningModule.useAutoStartListening);
const mockUseListening = vi.mocked(useListeningModule.useListening);
const mockUseRecording = vi.mocked(useRecordingModule.useRecording);
const mockUseMultiModelStatus = vi.mocked(useMultiModelStatusModule.useMultiModelStatus);
const mockUseSettings = vi.mocked(useSettingsModule.useSettings);

describe("App Integration", () => {
  const defaultAppStatusMock: useAppStatusModule.UseAppStatusResult = {
    status: "idle",
    isRecording: false,
    isTranscribing: false,
    isListening: false,
    error: null,
  };

  const defaultListeningMock: useListeningModule.UseListeningReturn = {
    isListening: false,
    isWakeWordDetected: false,
    isMicAvailable: true,
    error: null,
    enableListening: vi.fn(),
    disableListening: vi.fn(),
  };

  const defaultRecordingMock: useRecordingModule.UseRecordingResult = {
    isRecording: false,
    error: null,
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    lastRecording: null,
    wasCancelled: false,
    cancelReason: null,
  };

  const defaultMultiModelStatusMock: useMultiModelStatusModule.UseMultiModelStatusReturn = {
    models: {
      isAvailable: true,
      downloadState: "idle" as const,
      progress: 0,
      error: null,
    },
    downloadModel: vi.fn(),
    checkStatus: vi.fn(),
  };

  const defaultSettingsMock = {
    settings: {
      audio: { selectedDevice: null },
      listening: { enabled: false, autoStartOnLaunch: false },
    },
    updateSettings: vi.fn(),
    isLoading: false,
    error: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseCatOverlay.mockReturnValue({ isRecording: false });
    mockUseAppStatus.mockReturnValue(defaultAppStatusMock);
    mockUseAutoStartListening.mockReturnValue(undefined);
    mockUseListening.mockReturnValue(defaultListeningMock);
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
