import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TranscriptionSettings } from "./TranscriptionSettings";

// Mock invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock listen
const mockListeners: Map<string, ((event: { payload: unknown }) => void)[]> = new Map();
const mockListen = vi.fn().mockImplementation((eventName: string, callback: (event: { payload: unknown }) => void) => {
  const listeners = mockListeners.get(eventName) || [];
  listeners.push(callback);
  mockListeners.set(eventName, listeners);
  return Promise.resolve(() => {
    const currentListeners = mockListeners.get(eventName) || [];
    const index = currentListeners.indexOf(callback);
    if (index > -1) {
      currentListeners.splice(index, 1);
    }
  });
});

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

function emitEvent(eventName: string, payload: unknown) {
  const listeners = mockListeners.get(eventName) || [];
  listeners.forEach((cb) => cb({ payload }));
}

describe("TranscriptionSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListeners.clear();

    // Default mocks
    mockInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(false);
      }
      if (cmd === "get_transcription_mode") {
        return Promise.resolve("batch");
      }
      return Promise.resolve();
    });
  });

  describe("Component rendering", () => {
    it("renders with both model sections visible", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Transcription")).toBeDefined();
      });

      expect(screen.getByText("Batch (TDT)")).toBeDefined();
      expect(screen.getByText("Streaming (EOU)")).toBeDefined();
    });

    it("displays model descriptions", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("High-accuracy transcription after recording completes")).toBeDefined();
      });
      expect(screen.getByText("Real-time transcription as you speak")).toBeDefined();
    });

    it("applies custom className", async () => {
      const { container } = render(<TranscriptionSettings className="custom-class" />);

      await waitFor(() => {
        expect(container.querySelector(".transcription-settings.custom-class")).toBeDefined();
      });
    });
  });

  describe("Model download functionality", () => {
    it("TDT download button triggers download_model with model_type='tdt'", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");

      await userEvent.click(downloadButton!);

      expect(mockInvoke).toHaveBeenCalledWith("download_model", { modelType: "tdt" });
    });

    it("EOU download button triggers download_model with model_type='eou'", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Streaming (EOU)")).toBeDefined();
      });

      const eouCard = screen.getByText("Streaming (EOU)").closest(".transcription-settings__model-card");
      const downloadButton = eouCard?.querySelector("button");

      await userEvent.click(downloadButton!);

      expect(mockInvoke).toHaveBeenCalledWith("download_model", { modelType: "eou" });
    });

    it("progress bar updates when model_file_download_progress event is received", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      // Start download
      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      // Emit progress event
      emitEvent("model_file_download_progress", {
        model_type: "tdt",
        file_name: "model.bin",
        percent: 50,
        bytes_downloaded: 500,
        total_bytes: 1000,
      });

      await waitFor(() => {
        const progressBar = screen.getByRole("progressbar", { name: "Batch (TDT) download progress" });
        expect(progressBar).toBeDefined();
        expect(progressBar.getAttribute("aria-valuenow")).toBe("50");
      });
    });

    it("download completion updates button to 'Model Ready' state", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      // Start download
      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      // Emit completion event
      emitEvent("model_download_completed", {
        model_type: "tdt",
        model_path: "/path/to/model",
      });

      await waitFor(() => {
        expect(screen.getAllByText("Model Ready").length).toBeGreaterThan(0);
      });
    });

    it("error state displays error message below button", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "download_model") {
          return Promise.reject(new Error("Download failed: Network error"));
        }
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(false);
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      await waitFor(() => {
        expect(screen.getByText("Download failed: Network error")).toBeDefined();
      });
    });

    it("retry button appears after download error", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "download_model") {
          return Promise.reject(new Error("Network error"));
        }
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(false);
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      await waitFor(() => {
        expect(screen.getByText("Retry Download")).toBeDefined();
      });
    });
  });

  describe("Mode toggle functionality", () => {
    it("mode toggle is disabled when selected model is not available", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(false);
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Mode")).toBeDefined();
      });

      const batchRadio = screen.getByRole("radio", { name: /batch/i });
      const streamingRadio = screen.getByRole("radio", { name: /streaming/i });

      expect(batchRadio).toHaveProperty("disabled", true);
      expect(streamingRadio).toHaveProperty("disabled", true);
    });

    it("mode toggle is enabled when model is available", async () => {
      mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "check_parakeet_model_status") {
          const modelType = args?.modelType;
          return Promise.resolve(modelType === "ParakeetTDT");
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        const batchRadio = screen.getByRole("radio", { name: /batch/i });
        expect(batchRadio).toHaveProperty("disabled", false);
      });
    });

    it("mode toggle calls set_transcription_mode command on change", async () => {
      mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(true);
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        if (cmd === "set_transcription_mode") {
          return Promise.resolve();
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        const streamingRadio = screen.getByRole("radio", { name: /streaming/i });
        expect(streamingRadio).toHaveProperty("disabled", false);
      });

      const streamingRadio = screen.getByRole("radio", { name: /streaming/i });
      await userEvent.click(streamingRadio);

      expect(mockInvoke).toHaveBeenCalledWith("set_transcription_mode", { mode: "streaming" });
    });
  });

  describe("Model status check", () => {
    it("checks model status on component mount via check_parakeet_model_status command", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", { modelType: "ParakeetTDT" });
        expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", { modelType: "ParakeetEOU" });
      });
    });

    it("displays 'Model Ready' when model is already available", async () => {
      mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(true);
        }
        if (cmd === "get_transcription_mode") {
          return Promise.resolve("batch");
        }
        return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        const readyButtons = screen.getAllByText("Model Ready");
        expect(readyButtons.length).toBe(2);
      });
    });
  });

  describe("Accessibility", () => {
    it("has proper ARIA labels for model download controls", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      expect(screen.getByLabelText("Click to download Batch (TDT) model")).toBeDefined();
      expect(screen.getByLabelText("Click to download Streaming (EOU) model")).toBeDefined();
    });

    it("has proper role and aria-label for the settings region", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByRole("region", { name: "Transcription settings" })).toBeDefined();
      });
    });

    it("mode toggle has proper radiogroup role", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByRole("radiogroup", { name: "Transcription mode" })).toBeDefined();
      });
    });

    it("aria-busy is set during download", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      expect(downloadButton?.getAttribute("aria-busy")).toBe("true");
    });
  });
});
