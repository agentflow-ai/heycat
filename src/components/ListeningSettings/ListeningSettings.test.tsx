import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ListeningSettings } from "./ListeningSettings";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "list_audio_devices") {
      return Promise.resolve([
        { name: "Built-in Microphone", isDefault: true },
      ]);
    }
    return Promise.resolve(undefined);
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe("ListeningSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStore.get.mockResolvedValue(false);
  });

  it("renders the Listening title", async () => {
    render(<ListeningSettings />);

    expect(await screen.findByText("Listening")).toBeDefined();
  });

  it("renders the Always Listening section", async () => {
    render(<ListeningSettings />);

    expect(await screen.findByText("Always Listening")).toBeDefined();
  });

  it("renders Enable Listening Mode toggle", async () => {
    render(<ListeningSettings />);

    expect(await screen.findByText("Enable Listening Mode")).toBeDefined();
  });

  it("renders Auto-start on Launch toggle", async () => {
    render(<ListeningSettings />);

    expect(await screen.findByText("Auto-start on Launch")).toBeDefined();
  });

  it("toggle switches have correct aria-checked attribute when off", async () => {
    mockStore.get.mockResolvedValue(false);
    render(<ListeningSettings />);

    const switches = await screen.findAllByRole("switch");
    expect(switches).toHaveLength(2);
    expect(switches[0].getAttribute("aria-checked")).toBe("false");
    expect(switches[1].getAttribute("aria-checked")).toBe("false");
  });

  it("toggle switches have correct aria-checked attribute when on", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "listening.enabled") return Promise.resolve(true);
      if (key === "listening.autoStartOnLaunch") return Promise.resolve(true);
      return Promise.resolve(false);
    });
    render(<ListeningSettings />);

    const switches = await screen.findAllByRole("switch");
    expect(switches).toHaveLength(2);
    expect(switches[0].getAttribute("aria-checked")).toBe("true");
    expect(switches[1].getAttribute("aria-checked")).toBe("true");
  });

  it("clicking Enable Listening Mode toggle calls store.set", async () => {
    mockStore.get.mockResolvedValue(false);
    render(<ListeningSettings />);

    const switches = await screen.findAllByRole("switch");
    await userEvent.click(switches[0]);

    expect(mockStore.set).toHaveBeenCalledWith("listening.enabled", true);
  });

  it("clicking Auto-start toggle calls store.set", async () => {
    mockStore.get.mockResolvedValue(false);
    render(<ListeningSettings />);

    const switches = await screen.findAllByRole("switch");
    await userEvent.click(switches[1]);

    expect(mockStore.set).toHaveBeenCalledWith(
      "listening.autoStartOnLaunch",
      true
    );
  });

  it("applies custom className", async () => {
    render(<ListeningSettings className="custom-class" />);

    const container = await screen.findByRole("region", {
      name: "Listening settings",
    });
    expect(container.className).toContain("custom-class");
  });

  it("renders region with proper aria label", async () => {
    render(<ListeningSettings />);

    const region = await screen.findByRole("region", {
      name: "Listening settings",
    });
    expect(region).toBeDefined();
  });
});
