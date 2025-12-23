import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { AudioSettings } from "./AudioSettings";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock Tauri listen
const mockUnlisten = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(mockUnlisten),
}));

// Mock useSettings hook
const mockUpdateNoiseSuppression = vi.fn().mockResolvedValue(undefined);
const mockUpdateAudioDevice = vi.fn().mockResolvedValue(undefined);

vi.mock("../../hooks/useSettings", () => ({
  useSettings: () => ({
    settings: {
      listening: { enabled: false, autoStartOnLaunch: false },
      audio: { selectedDevice: null, noiseSuppression: true },
      shortcuts: { distinguishLeftRight: false },
    },
    isLoading: false,
    updateAudioDevice: mockUpdateAudioDevice,
    updateNoiseSuppression: mockUpdateNoiseSuppression,
    updateListeningEnabled: vi.fn(),
    updateAutoStartListening: vi.fn(),
    updateDistinguishLeftRight: vi.fn(),
  }),
}));

// Mock useAudioDevices hook
vi.mock("../../hooks/useAudioDevices", () => ({
  useAudioDevices: () => ({
    devices: [
      { name: "Default Microphone", isDefault: true },
      { name: "USB Microphone", isDefault: false },
    ],
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

// Mock useAudioLevelMonitor hook
vi.mock("../../hooks/useAudioLevelMonitor", () => ({
  useAudioLevelMonitor: () => ({
    level: 50,
    isMonitoring: true,
    error: null,
  }),
}));

// Mock useToast hook
const mockToast = vi.fn();
vi.mock("../../components/overlays", () => ({
  useToast: () => ({
    toast: mockToast,
  }),
}));

describe("AudioSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  describe("Noise Suppression Toggle", () => {
    it("renders noise suppression toggle with label", () => {
      render(<AudioSettings />);

      expect(screen.getByText("Noise Suppression")).toBeDefined();
      expect(screen.getByText("Reduce background noise during recording")).toBeDefined();
    });

    it("toggle is checked by default (noise suppression enabled)", () => {
      render(<AudioSettings />);

      const toggle = screen.getByRole("switch", { name: /noise suppression/i });
      expect(toggle).toHaveAttribute("data-state", "checked");
    });

    it("clicking toggle calls updateNoiseSuppression with false", async () => {
      const user = userEvent.setup();
      render(<AudioSettings />);

      const toggle = screen.getByRole("switch", { name: /noise suppression/i });
      await user.click(toggle);

      await waitFor(() => {
        expect(mockUpdateNoiseSuppression).toHaveBeenCalledWith(false);
      });
    });

    it("shows toast notification after toggling", async () => {
      const user = userEvent.setup();
      render(<AudioSettings />);

      const toggle = screen.getByRole("switch", { name: /noise suppression/i });
      await user.click(toggle);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          type: "success",
          title: "Setting saved",
          description: "Noise suppression disabled.",
        });
      });
    });
  });
});
