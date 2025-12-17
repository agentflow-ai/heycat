import { type ReactNode } from "react";
import { Header } from "./Header";
import { Sidebar, type NavItem } from "./Sidebar";
import { MainContent } from "./MainContent";
import { Footer } from "./Footer";

export interface AppShellProps {
  children: ReactNode;
  /** Currently active navigation item ID */
  activeNavItem?: string;
  /** Callback when navigation item is clicked */
  onNavigate?: (itemId: string) => void;
  /** Current status for the status pill */
  status?: "idle" | "listening" | "recording" | "processing";
  /** Status label override */
  statusLabel?: string;
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
}

const defaultNavItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "LayoutDashboard" },
  { id: "recordings", label: "Recordings", icon: "Mic" },
  { id: "commands", label: "Commands", icon: "MessageSquare" },
  { id: "settings", label: "Settings", icon: "Settings" },
];

export function AppShell({
  children,
  activeNavItem = "dashboard",
  onNavigate,
  status = "idle",
  statusLabel,
  footerStateDescription = "Ready for your command.",
  footerCenter,
  footerActions,
  onCommandPaletteOpen,
  onSettingsClick,
  onHelpClick,
}: AppShellProps) {
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
        onCommandPaletteOpen={onCommandPaletteOpen}
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
    </div>
  );
}
