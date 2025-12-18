import { useState } from "react";
import { GeneralSettings } from "./components/GeneralSettings";
import { AudioSettings } from "./components/AudioSettings";
import { TranscriptionTab } from "./components/TranscriptionTab";
import { AboutSettings } from "./components/AboutSettings";

export type SettingsTab = "general" | "audio" | "transcription" | "about";

export interface SettingsProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
  /** Initial tab to display (for URL routing) */
  initialTab?: SettingsTab;
}

const tabs: { id: SettingsTab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "audio", label: "Audio" },
  { id: "transcription", label: "Transcription" },
  { id: "about", label: "About" },
];

export function Settings({ onNavigate, initialTab = "general" }: SettingsProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab);

  const handleTabChange = (tab: SettingsTab) => {
    setActiveTab(tab);
  };

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">Settings</h1>
      </header>

      {/* Tab Navigation */}
      <nav className="border-b border-border" role="tablist" aria-label="Settings tabs">
        <div className="flex gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={activeTab === tab.id}
              aria-controls={`${tab.id}-panel`}
              id={`${tab.id}-tab`}
              onClick={() => handleTabChange(tab.id)}
              className={`
                px-4 py-2 text-sm font-medium rounded-t-md transition-colors
                focus:outline-none focus-visible:ring-2 focus-visible:ring-heycat-teal focus-visible:ring-offset-2
                ${
                  activeTab === tab.id
                    ? "text-heycat-orange border-b-2 border-heycat-orange bg-heycat-cream/50"
                    : "text-text-secondary hover:text-text-primary hover:bg-surface"
                }
              `}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </nav>

      {/* Tab Content */}
      <div className="mt-6">
        {activeTab === "general" && (
          <div
            role="tabpanel"
            id="general-panel"
            aria-labelledby="general-tab"
          >
            <GeneralSettings />
          </div>
        )}
        {activeTab === "audio" && (
          <div
            role="tabpanel"
            id="audio-panel"
            aria-labelledby="audio-tab"
          >
            <AudioSettings />
          </div>
        )}
        {activeTab === "transcription" && (
          <div
            role="tabpanel"
            id="transcription-panel"
            aria-labelledby="transcription-tab"
          >
            <TranscriptionTab />
          </div>
        )}
        {activeTab === "about" && (
          <div
            role="tabpanel"
            id="about-panel"
            aria-labelledby="about-tab"
          >
            <AboutSettings />
          </div>
        )}
      </div>
    </div>
  );
}
