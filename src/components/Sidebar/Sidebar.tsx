import { useState } from "react";
import { RecordingsList } from "../RecordingsView";
import { CommandSettings } from "../CommandSettings";
import "./Sidebar.css";

export type SidebarTab = "history" | "commands";

export interface SidebarProps {
  className?: string;
  defaultTab?: SidebarTab;
}

export function Sidebar({ className = "", defaultTab = "history" }: SidebarProps) {
  const [activeTab, setActiveTab] = useState<SidebarTab>(defaultTab);

  return (
    <aside className={`sidebar ${className}`.trim()} role="complementary">
      <nav className="sidebar__nav" role="tablist" aria-label="Sidebar navigation">
        <button
          className={`sidebar__tab ${activeTab === "history" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "history"}
          aria-controls="sidebar-panel-history"
          onClick={() => setActiveTab("history")}
          type="button"
        >
          History
        </button>
        <button
          className={`sidebar__tab ${activeTab === "commands" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "commands"}
          aria-controls="sidebar-panel-commands"
          onClick={() => setActiveTab("commands")}
          type="button"
        >
          Commands
        </button>
      </nav>
      <div
        id={`sidebar-panel-${activeTab}`}
        className="sidebar__panel"
        role="tabpanel"
        aria-labelledby={`sidebar-tab-${activeTab}`}
      >
        {activeTab === "history" && <RecordingsList />}
        {activeTab === "commands" && <CommandSettings />}
      </div>
    </aside>
  );
}
