import { useState } from "react";
import { RecordingsList } from "../RecordingsView";
import "./Sidebar.css";

export type SidebarTab = "history";

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
      </nav>
      <div
        id="sidebar-panel-history"
        className="sidebar__panel"
        role="tabpanel"
        aria-labelledby="sidebar-tab-history"
      >
        {activeTab === "history" && <RecordingsList />}
      </div>
    </aside>
  );
}
