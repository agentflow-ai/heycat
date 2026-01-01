import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Dashboard } from "./Dashboard";

// Mock Tauri events
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock hooks
const mockDownloadModel = vi.fn();

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

// Create wrapper for QueryClientProvider
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Dashboard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders dashboard with all sections", () => {
    render(<Dashboard />, { wrapper: createWrapper() });

    // Page header
    expect(screen.getByRole("heading", { name: "Dashboard" })).toBeDefined();
    expect(
      screen.getByText("Welcome back! Here's your HeyCat status.")
    ).toBeDefined();

    // Status cards
    expect(screen.getByText("Recordings")).toBeDefined();
    expect(screen.getByText("Commands")).toBeDefined();

    // Quick action buttons (recording button removed - only hotkey-based recording)
    expect(screen.getByRole("button", { name: "Train Command" })).toBeDefined();
    expect(
      screen.getByRole("button", { name: "Download Model" })
    ).toBeDefined();
  });

  it("download model button triggers download", async () => {
    const user = userEvent.setup();
    render(<Dashboard />, { wrapper: createWrapper() });

    await user.click(screen.getByRole("button", { name: "Download Model" }));
    expect(mockDownloadModel).toHaveBeenCalledWith("tdt");
  });

  it("navigation cards trigger onNavigate callback", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    render(<Dashboard onNavigate={handleNavigate} />, { wrapper: createWrapper() });

    // Click recordings card
    const recordingsCard = screen.getByText("Recordings").closest("[role=button]");
    await user.click(recordingsCard!);
    expect(handleNavigate).toHaveBeenCalledWith("recordings");

    // Click commands card
    const commandsCard = screen.getByText("Commands").closest("[role=button]");
    await user.click(commandsCard!);
    expect(handleNavigate).toHaveBeenCalledWith("commands");
  });
});
