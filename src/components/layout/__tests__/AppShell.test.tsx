import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "../AppShell";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("AppShell", () => {
  it("renders layout structure with all sections", () => {
    render(
      <AppShell>
        <div>Page Content</div>
      </AppShell>
    );

    // Header with logo
    expect(screen.getByRole("banner")).toBeDefined();
    expect(screen.getByText("HeyCat")).toBeDefined();

    // Navigation sidebar (labeled)
    expect(screen.getByRole("navigation", { name: /main navigation/i })).toBeDefined();
    expect(screen.getByText("Dashboard")).toBeDefined();
    expect(screen.getByText("Recordings")).toBeDefined();
    expect(screen.getByText("Commands")).toBeDefined();
    expect(screen.getByText("Settings")).toBeDefined();

    // Main content area
    expect(screen.getByRole("main")).toBeDefined();
    expect(screen.getByText("Page Content")).toBeDefined();

    // Footer
    expect(screen.getByRole("contentinfo")).toBeDefined();
    expect(screen.getByText("Ready for your command.")).toBeDefined();
  });

  it("highlights active navigation item and allows navigation", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    render(
      <AppShell activeNavItem="dashboard" onNavigate={handleNavigate}>
        <div>Content</div>
      </AppShell>
    );

    // Dashboard should be active (has aria-current)
    const dashboardBtn = screen.getByText("Dashboard").closest("button");
    expect(dashboardBtn?.getAttribute("aria-current")).toBe("page");

    // Recordings should not be active
    const recordingsBtn = screen.getByText("Recordings").closest("button");
    expect(recordingsBtn?.getAttribute("aria-current")).toBeNull();

    // Click Recordings to navigate
    await user.click(recordingsBtn!);
    expect(handleNavigate).toHaveBeenCalledWith("recordings");
  });

  it("displays different status states in header", () => {
    const { rerender } = render(
      <AppShell status="idle">
        <div>Content</div>
      </AppShell>
    );
    // StatusPill shows "Ready" for idle state per ui.md spec
    expect(screen.getByText("Ready")).toBeDefined();

    rerender(
      <AppShell status="recording">
        <div>Content</div>
      </AppShell>
    );
    expect(screen.getByText("Recording")).toBeDefined();

    rerender(
      <AppShell status="processing">
        <div>Content</div>
      </AppShell>
    );
    expect(screen.getByText("Processing...")).toBeDefined();
  });

  it("triggers header action callbacks", async () => {
    const user = userEvent.setup();
    const handleCommandPalette = vi.fn();
    const handleSettings = vi.fn();
    const handleHelp = vi.fn();

    render(
      <AppShell
        onCommandPaletteOpen={handleCommandPalette}
        onSettingsClick={handleSettings}
        onHelpClick={handleHelp}
      >
        <div>Content</div>
      </AppShell>
    );

    // Command palette (⌘K)
    await user.click(screen.getByLabelText(/command palette/i));
    expect(handleCommandPalette).toHaveBeenCalledTimes(1);

    // Settings
    await user.click(screen.getByLabelText("Settings"));
    expect(handleSettings).toHaveBeenCalledTimes(1);

    // Help
    await user.click(screen.getByLabelText("Help"));
    expect(handleHelp).toHaveBeenCalledTimes(1);
  });

  it("displays custom footer content", () => {
    render(
      <AppShell
        footerStateDescription="Currently listening..."
        footerCenter={<div data-testid="audio-meter">Audio Level</div>}
        footerActions={<button type="button">Stop</button>}
      >
        <div>Content</div>
      </AppShell>
    );

    expect(screen.getByText("Currently listening...")).toBeDefined();
    expect(screen.getByTestId("audio-meter")).toBeDefined();
    expect(screen.getByRole("button", { name: "Stop" })).toBeDefined();
  });

  it("navigates when command palette navigation command is selected", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    render(
      <AppShell activeNavItem="dashboard" onNavigate={handleNavigate}>
        <div>Content</div>
      </AppShell>
    );

    // Open command palette with Cmd+K
    await user.keyboard("{Meta>}k{/Meta}");

    // Wait for palette to open and click a navigation command
    const recordingsCommand = await screen.findByText("Go to Recordings");
    await user.click(recordingsCommand);

    expect(handleNavigate).toHaveBeenCalledWith("recordings");
  });

  it("triggers recording actions from command palette", async () => {
    const user = userEvent.setup();
    const handleStartRecording = vi.fn();
    const handleStopRecording = vi.fn();

    render(
      <AppShell
        isRecording={false}
        onStartRecording={handleStartRecording}
        onStopRecording={handleStopRecording}
      >
        <div>Content</div>
      </AppShell>
    );

    // Open command palette and click Start Recording
    await user.keyboard("{Meta>}k{/Meta}");
    const startCommand = await screen.findByText("Start Recording");
    await user.click(startCommand);

    expect(handleStartRecording).toHaveBeenCalledTimes(1);
    expect(handleStopRecording).not.toHaveBeenCalled();
  });

  it("navigates to settings for audio device command", async () => {
    const user = userEvent.setup();
    const handleNavigate = vi.fn();

    render(
      <AppShell onNavigate={handleNavigate}>
        <div>Content</div>
      </AppShell>
    );

    // Open command palette and click Change Audio Device
    await user.keyboard("{Meta>}k{/Meta}");
    const audioCommand = await screen.findByText("Change Audio Device");
    await user.click(audioCommand);

    expect(handleNavigate).toHaveBeenCalledWith("settings");
  });
});
