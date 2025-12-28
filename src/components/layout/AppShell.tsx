import { type ReactNode, useCallback } from "react";
import { Header } from "./Header";
import { Sidebar, type NavItem } from "./Sidebar";
import { MainContent } from "./MainContent";
import { Footer } from "./Footer";
import { CommandPalette, useCommandPalette } from "../overlays";

export interface AppShellProps {
  children: ReactNode;
  /** Currently active navigation item ID */
  activeNavItem?: string;
  /** Callback when navigation item is clicked */
  onNavigate?: (itemId: string) => void;
  /** Current status for the status pill */
  status?: "idle" | "recording" | "processing";
  /** Status label override */
  statusLabel?: string;
  /** Recording duration in seconds (shown when status is recording) */
  recordingDuration?: number;
  /** Footer left section content (state description) */
  footerStateDescription?: string;
  /** Footer center content (audio meter) */
  footerCenter?: ReactNode;
  /** Footer right content (quick actions) */
  footerActions?: ReactNode;
  /** Callback when command palette trigger is clicked */
  onCommandPaletteOpen?: () => void;
  /** Callback when settings is clicked */
  onSettingsClick?: () => void;
  /** Callback when help is clicked */
  onHelpClick?: () => void;
  /** Whether recording is currently active */
  isRecording?: boolean;
  /** Callback to start recording */
  onStartRecording?: () => void;
  /** Callback to stop recording */
  onStopRecording?: () => void;
}

const defaultNavItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "LayoutDashboard" },
  { id: "recordings", label: "Recordings", icon: "Mic" },
  { id: "commands", label: "Commands", icon: "MessageSquare" },
  { id: "dictionary", label: "Dictionary", icon: "Book" },
  { id: "contexts", label: "Contexts", icon: "Layers" },
  { id: "settings", label: "Settings", icon: "Settings" },
];

export function AppShell({
  children,
  activeNavItem = "dashboard",
  onNavigate,
  status = "idle",
  statusLabel,
  recordingDuration,
  footerStateDescription = "Ready for your command.",
  footerCenter,
  footerActions,
  onCommandPaletteOpen,
  onSettingsClick,
  onHelpClick,
  isRecording = false,
  onStartRecording,
  onStopRecording,
}: AppShellProps) {
  const { isOpen, open, close } = useCommandPalette({
    onOpen: onCommandPaletteOpen,
  });

  const handleCommandExecute = useCallback(
    (commandId: string) => {
      switch (commandId) {
        // Navigation commands
        case "go-dashboard":
          onNavigate?.("dashboard");
          break;
        case "go-recordings":
          onNavigate?.("recordings");
          break;
        case "go-commands":
          onNavigate?.("commands");
          break;
        case "go-settings":
          onNavigate?.("settings");
          break;
        // Recording commands
        case "start-recording":
          if (!isRecording) {
            onStartRecording?.();
          }
          break;
        case "stop-recording":
          if (isRecording) {
            onStopRecording?.();
          }
          break;
        // Settings commands - navigate to settings page
        case "change-audio-device":
        case "download-model":
          onNavigate?.("settings");
          break;
        // Help commands - navigate to settings or show help
        case "view-shortcuts":
        case "about-heycat":
          onHelpClick?.();
          break;
      }
    },
    [
      onNavigate,
      isRecording,
      onStartRecording,
      onStopRecording,
      onHelpClick,
    ]
  );

  return (
    <div
      className="h-screen w-screen flex flex-col bg-background"
      style={{
        boxShadow: "var(--shadow-window)",
      }}
    >
      <Header
        status={status}
        statusLabel={statusLabel}
        recordingDuration={recordingDuration}
        onCommandPaletteOpen={open}
        onSettingsClick={onSettingsClick}
        onHelpClick={onHelpClick}
      />
      <div className="flex flex-1 min-h-0">
        <Sidebar
          items={defaultNavItems}
          activeItemId={activeNavItem}
          onItemClick={onNavigate}
        />
        <div className="flex flex-col flex-1 min-w-0">
          <MainContent>{children}</MainContent>
          <Footer
            stateDescription={footerStateDescription}
            center={footerCenter}
            actions={footerActions}
          />
        </div>
      </div>
      <CommandPalette
        isOpen={isOpen}
        onClose={close}
        onCommandExecute={handleCommandExecute}
      />
    </div>
  );
}
