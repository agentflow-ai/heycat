import { render, screen } from "@testing-library/react";
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
const mockUpdateAudioDevice = vi.fn().mockResolvedValue(undefined);

vi.mock("../../hooks/useSettings", () => ({
  useSettings: () => ({
    settings: {
      listening: { enabled: false, autoStartOnLaunch: false },
      audio: { selectedDevice: null },
      shortcuts: { distinguishLeftRight: false },
    },
    isLoading: false,
    updateAudioDevice: mockUpdateAudioDevice,
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

  describe("Audio Input Section", () => {
    it("renders audio input section with device selector", () => {
      render(<AudioSettings />);

      expect(screen.getByText("Audio Input")).toBeDefined();
      expect(screen.getByText("Input Device")).toBeDefined();
    });

    it("renders audio level meter", () => {
      render(<AudioSettings />);

      expect(screen.getByText("Audio Level")).toBeDefined();
      expect(screen.getByText("Good")).toBeDefined(); // Based on level=50
    });
  });
});
