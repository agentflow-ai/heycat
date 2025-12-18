/* v8 ignore file -- @preserve */
import { useState, useEffect } from "react";
import { AppShell } from "./components/layout/AppShell";
import { ToastProvider } from "./components/overlays";
import { Dashboard, Commands, Recordings, Settings } from "./pages";
import { useCatOverlay } from "./hooks/useCatOverlay";
import { useAutoStartListening } from "./hooks/useAutoStartListening";
import { useAppStatus } from "./hooks/useAppStatus";

function App() {
  const { status: appStatus, isRecording } = useAppStatus();
  const [navItem, setNavItem] = useState("dashboard");
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
        {navItem === "recordings" && <Recordings onNavigate={setNavItem} />}
        {navItem === "settings" && <Settings onNavigate={setNavItem} />}
        {navItem !== "dashboard" && navItem !== "commands" && navItem !== "recordings" && navItem !== "settings" && (
          <div className="flex items-center justify-center h-full text-text-secondary">
            <p>Page coming soon</p>
          </div>
        )}
      </AppShell>
    </ToastProvider>
  );
}

export default App;
