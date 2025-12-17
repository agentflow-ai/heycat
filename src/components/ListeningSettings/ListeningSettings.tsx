import { useSettings } from "../../hooks/useSettings";
import { useListening } from "../../hooks/useListening";
import { AudioDeviceSelector } from "./AudioDeviceSelector";
import "./ListeningSettings.css";

export interface ListeningSettingsProps {
  className?: string;
}

export function ListeningSettings({ className = "" }: ListeningSettingsProps) {
  const {
    settings,
    isLoading,
    error,
    updateListeningEnabled,
    updateAutoStartListening,
  } = useSettings();
  const { isListening, enableListening, disableListening } = useListening();

  const handleToggleListening = async () => {
    const newEnabled = !settings.listening.enabled;
    await updateListeningEnabled(newEnabled);

    // Sync with backend
    if (newEnabled) {
      await enableListening();
    } else {
      await disableListening();
    }
  };

  const handleToggleAutoStart = async () => {
    await updateAutoStartListening(!settings.listening.autoStartOnLaunch);
  };

  if (isLoading) {
    return (
      <div
        className={`listening-settings ${className}`.trim()}
        role="region"
        aria-label="Listening settings"
      >
        <div className="listening-settings__loading">Loading settings...</div>
      </div>
    );
  }

  return (
    <div
      className={`listening-settings ${className}`.trim()}
      role="region"
      aria-label="Listening settings"
    >
      <div className="listening-settings__header">
        <h2 className="listening-settings__title">Listening</h2>
      </div>

      {error && (
        <div className="listening-settings__error" role="alert">
          {error}
        </div>
      )}

      <section className="listening-settings__section">
        <h3 className="listening-settings__section-title">Audio Input</h3>
        <AudioDeviceSelector />
      </section>

      <section className="listening-settings__section">
        <h3 className="listening-settings__section-title">Always Listening</h3>

        <div className="listening-settings__toggle-group">
          <label className="listening-settings__toggle">
            <div className="listening-settings__toggle-info">
              <span className="listening-settings__toggle-label">
                Enable Listening Mode
              </span>
              <span className="listening-settings__toggle-description">
                Listen for wake word to activate voice commands
              </span>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={settings.listening.enabled}
              className={`listening-settings__switch ${
                settings.listening.enabled
                  ? "listening-settings__switch--on"
                  : ""
              }`.trim()}
              onClick={handleToggleListening}
            >
              <span className="listening-settings__switch-thumb" />
            </button>
          </label>

          <label className="listening-settings__toggle">
            <div className="listening-settings__toggle-info">
              <span className="listening-settings__toggle-label">
                Auto-start on Launch
              </span>
              <span className="listening-settings__toggle-description">
                Begin listening automatically when the app starts
              </span>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={settings.listening.autoStartOnLaunch}
              className={`listening-settings__switch ${
                settings.listening.autoStartOnLaunch
                  ? "listening-settings__switch--on"
                  : ""
              }`.trim()}
              onClick={handleToggleAutoStart}
            >
              <span className="listening-settings__switch-thumb" />
            </button>
          </label>
        </div>
      </section>

      {isListening && (
        <div className="listening-settings__status listening-settings__status--active">
          Listening mode is active
        </div>
      )}
    </div>
  );
}
