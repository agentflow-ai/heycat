/* v8 ignore file -- @preserve */
import "./App.css";
import { RecordingIndicator } from "./components/RecordingIndicator";
import { TranscriptionIndicator } from "./components/TranscriptionIndicator";
import { TranscriptionNotification } from "./components/TranscriptionNotification";
import { Sidebar } from "./components/Sidebar";
import { ModelDownloadButton } from "./components/ModelDownloadButton";
import { useTranscription } from "./hooks/useTranscription";

function App() {
  const { isTranscribing } = useTranscription();

  return (
    <div className="app-layout">
      <Sidebar className="app-sidebar" />
      <main className="container">
        <RecordingIndicator className="app-recording-indicator" isBlocked={isTranscribing} />
        <TranscriptionIndicator className="app-transcription-indicator" />
        <ModelDownloadButton className="app-model-download" />
        <TranscriptionNotification className="app-transcription-notification" />
      </main>
    </div>
  );
}

export default App;
