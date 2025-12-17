import { useAudioDevices } from "../../hooks/useAudioDevices";
import { useSettings } from "../../hooks/useSettings";
import "./AudioDeviceSelector.css";

export function AudioDeviceSelector() {
  const { devices, isLoading, error, refresh } = useAudioDevices();
  const { settings, updateAudioDevice } = useSettings();

  const selectedDevice = settings.audio.selectedDevice;

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const value = event.target.value;
    updateAudioDevice(value === "" ? null : value);
  };

  if (isLoading) {
    return (
      <div className="audio-device-selector audio-device-selector--loading">
        Loading devices...
      </div>
    );
  }

  if (error) {
    return (
      <div className="audio-device-selector audio-device-selector--error">
        <span>Failed to load devices</span>
        <button
          type="button"
          onClick={refresh}
          className="audio-device-selector__retry-button"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="audio-device-selector">
      <label
        htmlFor="audio-device-select"
        className="audio-device-selector__label"
      >
        Microphone
      </label>
      <select
        id="audio-device-select"
        className="audio-device-selector__select"
        value={selectedDevice ?? ""}
        onChange={handleChange}
      >
        <option value="">System Default</option>
        {devices.map((device) => (
          <option key={device.name} value={device.name}>
            {device.name}
            {device.isDefault ? " (Default)" : ""}
          </option>
        ))}
      </select>
    </div>
  );
}
