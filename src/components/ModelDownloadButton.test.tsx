import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ModelDownloadButton } from "./ModelDownloadButton";
import * as useModelStatusModule from "../hooks/useModelStatus";

vi.mock("../hooks/useModelStatus");

const mockUseModelStatus = vi.mocked(useModelStatusModule.useModelStatus);

describe("ModelDownloadButton", () => {
  const defaultMock: useModelStatusModule.UseModelStatusResult = {
    isModelAvailable: false,
    downloadState: "idle",
    error: null,
    downloadModel: vi.fn(),
    refreshStatus: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseModelStatus.mockReturnValue(defaultMock);
  });

  it("renders download button in idle state", () => {
    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.textContent).toBe("Download Model");
    expect(button.disabled).toBe(false);
  });

  it("shows downloading state with spinner", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadState: "downloading",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.textContent).toContain("Downloading...");
    expect(button.disabled).toBe(true);
    expect(button.getAttribute("aria-busy")).toBe("true");
    expect(document.querySelector(".model-download-button__spinner")).toBeTruthy();
  });

  it("shows ready state when model is available", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      isModelAvailable: true,
      downloadState: "completed",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.textContent).toBe("Model Ready");
    expect(button.disabled).toBe(true);
    expect(button.className).toContain("model-download-button--ready");
  });

  it("shows error state with retry option", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadState: "error",
      error: "Network error",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.textContent).toBe("Retry Download");
    expect(button.disabled).toBe(false);
    expect(button.className).toContain("model-download-button--error");
    expect(screen.getByRole("alert").textContent).toBe("Network error");
  });

  it("calls downloadModel when button is clicked", () => {
    const mockDownload = vi.fn();
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadModel: mockDownload,
    });

    render(<ModelDownloadButton />);

    fireEvent.click(screen.getByRole("button"));
    expect(mockDownload).toHaveBeenCalledTimes(1);
  });

  it("applies custom className", () => {
    render(<ModelDownloadButton className="custom-class" />);

    const container = screen.getByRole("region");
    expect(container.className).toContain("custom-class");
  });

  it("has correct aria-label in idle state", () => {
    render(<ModelDownloadButton />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe(
      "Click to download whisper model"
    );
  });

  it("has correct aria-label when downloading", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadState: "downloading",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe(
      "Downloading whisper model, please wait"
    );
  });

  it("has correct aria-label when ready", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      isModelAvailable: true,
      downloadState: "completed",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe("Whisper model is ready");
  });

  it("has correct aria-label on error", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadState: "error",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe(
      "Download failed, click to retry"
    );
  });

  it("does not show error when there is none", () => {
    render(<ModelDownloadButton />);

    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("disables button when already completed", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      downloadState: "completed",
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("disables button when model is already available", () => {
    mockUseModelStatus.mockReturnValue({
      ...defaultMock,
      isModelAvailable: true,
    });

    render(<ModelDownloadButton />);

    const button = screen.getByRole("button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });
});
