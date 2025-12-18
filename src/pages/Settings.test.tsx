import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { Settings } from "./Settings";

// Mock Tauri invoke
const mockInvoke = vi.fn().mockImplementation((command: string) => {
  if (command === "get_recording_shortcut") {
    return Promise.resolve("CmdOrControl+Shift+R");
  }
  return Promise.resolve(undefined);
});
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock Tauri events
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock Tauri store plugin
vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(() => Promise.resolve(null)),
      set: vi.fn(() => Promise.resolve()),
    })
  ),
}));

// Mock hooks
const mockUpdateAutoStartListening = vi.fn();
const mockUpdateAudioDevice = vi.fn();
const mockDownloadModel = vi.fn();
const mockRefreshStatus = vi.fn();
const mockRefreshDevices = vi.fn();

vi.mock("../hooks/useSettings", () => ({
  useSettings: () => ({
    settings: {
      listening: {
        enabled: false,
        autoStartOnLaunch: false,
      },
      audio: {
        selectedDevice: null,
      },
    },
    isLoading: false,
    error: null,
    updateListeningEnabled: vi.fn(),
    updateAutoStartListening: mockUpdateAutoStartListening,
    updateAudioDevice: mockUpdateAudioDevice,
  }),
}));

vi.mock("../hooks/useAudioDevices", () => ({
  useAudioDevices: () => ({
    devices: [
      { name: "MacBook Pro Microphone", isDefault: true },
      { name: "External USB Microphone", isDefault: false },
    ],
    isLoading: false,
    error: null,
    refresh: mockRefreshDevices,
  }),
}));

vi.mock("../hooks/useAudioLevelMonitor", () => ({
  useAudioLevelMonitor: () => ({
    level: 45,
    isMonitoring: true,
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
    refreshStatus: mockRefreshStatus,
  }),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock("../components/overlays", () => ({
  useToast: () => ({
    toast: mockToast,
    dismiss: vi.fn(),
    dismissAll: vi.fn(),
  }),
}));

describe("Settings Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Tab Navigation", () => {
    it("renders settings page with all tabs", () => {
      render(<Settings />);

      expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
      expect(screen.getByRole("tab", { name: "General" })).toBeDefined();
      expect(screen.getByRole("tab", { name: "Audio" })).toBeDefined();
      expect(screen.getByRole("tab", { name: "Transcription" })).toBeDefined();
      expect(screen.getByRole("tab", { name: "About" })).toBeDefined();
    });

    it("shows General tab content by default", () => {
      render(<Settings />);

      // General tab should be active
      const generalTab = screen.getByRole("tab", { name: "General" });
      expect(generalTab.getAttribute("aria-selected")).toBe("true");

      // General settings should be visible
      expect(screen.getByText("Launch at Login")).toBeDefined();
      expect(screen.getByText("Auto-start Listening")).toBeDefined();
      expect(screen.getByText("Keyboard Shortcuts")).toBeDefined();
    });

    it("switches to Audio tab when clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Audio" }));

      // Audio tab should be active
      const audioTab = screen.getByRole("tab", { name: "Audio" });
      expect(audioTab.getAttribute("aria-selected")).toBe("true");

      // Audio settings should be visible
      expect(screen.getByText("Input Device")).toBeDefined();
      expect(screen.getByText("Wake Word")).toBeDefined();
    });

    it("switches to Transcription tab when clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Transcription" }));

      // Transcription settings should be visible
      expect(screen.getByText("Batch Model (TDT)")).toBeDefined();
    });

    it("switches to About tab when clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "About" }));

      // About content should be visible
      expect(screen.getByText("HeyCat")).toBeDefined();
      expect(screen.getByText(/Version/)).toBeDefined();
      expect(screen.getByText("GitHub Repository")).toBeDefined();
    });

    it("keeps tab state internal without triggering navigation", async () => {
      const user = userEvent.setup();
      const handleNavigate = vi.fn();
      render(<Settings onNavigate={handleNavigate} />);

      await user.click(screen.getByRole("tab", { name: "Audio" }));

      // Tab changes should NOT trigger navigation (stays on settings page)
      expect(handleNavigate).not.toHaveBeenCalled();
      // But the tab content should change
      expect(screen.getByText("Input Device")).toBeDefined();
    });
  });

  describe("General Tab", () => {
    it("toggles auto-start listening and shows toast", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      const autoStartToggle = screen.getByLabelText(/Auto-start Listening/i);
      await user.click(autoStartToggle);

      expect(mockUpdateAutoStartListening).toHaveBeenCalledWith(true);
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "success",
          title: "Setting saved",
        })
      );
    });

    it("opens shortcut editor when Change button is clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("button", { name: "Change" }));

      expect(screen.getByText("Change Keyboard Shortcut")).toBeDefined();
      expect(screen.getByText(/Set a new shortcut for/)).toBeDefined();
    });
  });

  describe("Audio Tab", () => {
    it("displays audio device selection", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Audio" }));

      expect(screen.getByText("Input Device")).toBeDefined();
      expect(screen.getByRole("button", { name: /Refresh/i })).toBeDefined();
    });

    it("shows audio level meter", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Audio" }));

      expect(screen.getByText("Audio Level")).toBeDefined();
      expect(screen.getByRole("meter", { name: /Audio level/i })).toBeDefined();
    });

    it("refreshes devices when Refresh button is clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Audio" }));
      await user.click(screen.getByRole("button", { name: /Refresh/i }));

      expect(mockRefreshDevices).toHaveBeenCalled();
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "info",
          title: "Refreshing devices",
        })
      );
    });
  });

  describe("Transcription Tab", () => {
    it("shows model status and download button when not installed", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Transcription" }));

      expect(screen.getByText("Batch Model (TDT)")).toBeDefined();
      expect(screen.getByText("Not Installed")).toBeDefined();
      expect(
        screen.getByRole("button", { name: /Download Model/i })
      ).toBeDefined();
    });

    it("triggers model download when button is clicked", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "Transcription" }));
      await user.click(screen.getByRole("button", { name: /Download Model/i }));

      expect(mockDownloadModel).toHaveBeenCalledWith("tdt");
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "info",
          title: "Download started",
        })
      );
    });
  });

  describe("About Tab", () => {
    it("displays app information and links", async () => {
      const user = userEvent.setup();
      render(<Settings />);

      await user.click(screen.getByRole("tab", { name: "About" }));

      expect(screen.getByText("HeyCat")).toBeDefined();
      expect(screen.getByText(/Version/)).toBeDefined();
      expect(screen.getByText("GitHub Repository")).toBeDefined();
      expect(screen.getByText("Documentation")).toBeDefined();
      expect(screen.getByText("Report an Issue")).toBeDefined();
      expect(screen.getByText("Acknowledgments")).toBeDefined();
    });
  });
});
