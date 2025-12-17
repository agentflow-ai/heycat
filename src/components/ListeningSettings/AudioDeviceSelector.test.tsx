import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AudioDeviceSelector } from "./AudioDeviceSelector";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

// Mock store for useSettings
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

describe("AudioDeviceSelector", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStore.get.mockResolvedValue(null);
    mockInvoke.mockResolvedValue([
      { name: "Built-in Microphone", isDefault: true },
      { name: "USB Microphone", isDefault: false },
    ]);
  });

  it("shows loading state initially", () => {
    mockInvoke.mockReturnValue(new Promise(() => {}));
    render(<AudioDeviceSelector />);

    expect(screen.getByText("Loading devices...")).toBeDefined();
  });

  it("renders device list after loading", async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByLabelText("Microphone")).toBeDefined();
    });

    const select = screen.getByRole("combobox");
    expect(select).toBeDefined();

    // Check options are present
    expect(screen.getByText("System Default")).toBeDefined();
    expect(screen.getByText("Built-in Microphone (Default)")).toBeDefined();
    expect(screen.getByText("USB Microphone")).toBeDefined();
  });

  it("shows System Default option", async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByLabelText("Microphone")).toBeDefined();
    });

    const options = screen.getAllByRole("option");
    expect(options[0]).toHaveProperty("value", "");
    expect(options[0].textContent).toBe("System Default");
  });

  it("marks default device with (Default) indicator", async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(
        screen.getByText("Built-in Microphone (Default)")
      ).toBeDefined();
    });
  });

  it("shows current selection from settings", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "audio.selectedDevice")
        return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    render(<AudioDeviceSelector />);

    await waitFor(() => {
      const select = screen.getByRole("combobox") as HTMLSelectElement;
      expect(select.value).toBe("USB Microphone");
    });
  });

  it("updates settings when selection changes", async () => {
    const user = userEvent.setup();
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByLabelText("Microphone")).toBeDefined();
    });

    const select = screen.getByRole("combobox");
    await user.selectOptions(select, "USB Microphone");

    expect(mockStore.set).toHaveBeenCalledWith(
      "audio.selectedDevice",
      "USB Microphone"
    );
  });

  it("clears selection when System Default is chosen", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "audio.selectedDevice")
        return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    const user = userEvent.setup();
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByLabelText("Microphone")).toBeDefined();
    });

    const select = screen.getByRole("combobox");
    await user.selectOptions(select, "");

    expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", null);
  });

  it("shows error state when device fetch fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Failed to list devices"));
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByText("Failed to load devices")).toBeDefined();
    });

    expect(screen.getByRole("button", { name: "Retry" })).toBeDefined();
  });

  it("retry button refetches devices", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Failed to list devices"));
    mockInvoke.mockResolvedValueOnce([
      { name: "Built-in Microphone", isDefault: true },
    ]);

    const user = userEvent.setup();
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByText("Failed to load devices")).toBeDefined();
    });

    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Microphone")).toBeDefined();
    });

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });
});
