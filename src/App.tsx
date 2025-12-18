/* v8 ignore file -- @preserve */
import { useState, useCallback, useEffect } from "react";
import "./App.css";
import { RecordingIndicator } from "./components/RecordingIndicator";
import { TranscriptionIndicator } from "./components/TranscriptionIndicator";
import { TranscriptionNotification } from "./components/TranscriptionNotification";
import { AudioErrorDialog } from "./components/AudioErrorDialog";
import { Sidebar, SidebarTab } from "./components/Sidebar";
import { AppShell } from "./components/layout/AppShell";
import { ToastProvider } from "./components/overlays";
import { UIToggle } from "./components/dev";
import { Dashboard, Commands } from "./pages";
import { useTranscription } from "./hooks/useTranscription";
import { useCatOverlay } from "./hooks/useCatOverlay";
import { useAutoStartListening } from "./hooks/useAutoStartListening";
import { useAudioErrorHandler } from "./hooks/useAudioErrorHandler";
import { useRecording } from "./hooks/useRecording";
import { useSettings } from "./hooks/useSettings";
import { useUIMode } from "./hooks/useUIMode";
import { useAppStatus } from "./hooks/useAppStatus";

function App() {
  const { settings } = useSettings();
  const { isTranscribing } = useTranscription();
  const { error: audioError, clearError } = useAudioErrorHandler();
  const { startRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });
  const { status: appStatus, isRecording } = useAppStatus();
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("history");
  const [navItem, setNavItem] = useState("dashboard");
  const { mode, toggle } = useUIMode();
  const [recordingDuration, setRecordingDuration] = useState(0);
  useCatOverlay();
  useAutoStartListening();

  // Track recording duration
  useEffect(() => {
    if (!isRecording) {
      setRecordingDuration(0);
      return;
    }
    setRecordingDuration(0);
    const interval = setInterval(() => {
      setRecordingDuration((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [isRecording]);

  const handleRetry = useCallback(() => {
    clearError();
    startRecording();
  }, [clearError, startRecording]);

  const handleSelectDevice = useCallback(() => {
    clearError();
    // Navigate to the Listening tab where device selection is available
    setSidebarTab("listening");
  }, [clearError]);

  // New UI mode - render AppShell layout
  if (mode === "new") {
    return (
      <ToastProvider>
        <AppShell
          activeNavItem={navItem}
          onNavigate={setNavItem}
          status={appStatus}
          recordingDuration={isRecording ? recordingDuration : undefined}
          footerStateDescription="Ready for your command."
        >
          {navItem === "dashboard" && <Dashboard onNavigate={setNavItem} />}
          {navItem === "commands" && <Commands onNavigate={setNavItem} />}
          {navItem !== "dashboard" && navItem !== "commands" && (
            <div className="flex items-center justify-center h-full text-text-secondary">
              <p>Page coming soon</p>
            </div>
          )}
        </AppShell>
        <UIToggle mode={mode} onToggle={toggle} />
      </ToastProvider>
    );
  }

  // Old UI mode - render existing Sidebar-based layout
  return (
    <>
      <div className="app-layout">
        <Sidebar className="app-sidebar" activeTab={sidebarTab} onTabChange={setSidebarTab} />
        <main className="container">
          <RecordingIndicator className="app-recording-indicator" isBlocked={isTranscribing} />
          <TranscriptionIndicator className="app-transcription-indicator" />
          <TranscriptionNotification className="app-transcription-notification" />
        </main>
        <AudioErrorDialog
          error={audioError}
          onRetry={handleRetry}
          onSelectDevice={handleSelectDevice}
          onDismiss={clearError}
        />
      </div>
      <UIToggle mode={mode} onToggle={toggle} />
    </>
  );
}

export default App;
