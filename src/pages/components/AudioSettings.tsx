import { useState } from "react";
import { RefreshCw } from "lucide-react";
import {
  Card,
  CardContent,
  Button,
  Select,
  SelectItem,
  AudioLevelMeter,
} from "../../components/ui";
import { useSettings } from "../../hooks/useSettings";
import { useAudioDevices } from "../../hooks/useAudioDevices";
import { useAudioLevelMonitor } from "../../hooks/useAudioLevelMonitor";
import { useToast } from "../../components/overlays";

export interface AudioSettingsProps {
  className?: string;
}

type Sensitivity = "low" | "medium" | "high";

function getLevelIndicator(level: number): { text: string; color: string } {
  if (level < 10) return { text: "Low", color: "text-text-secondary" };
  if (level > 85) return { text: "High", color: "text-error" };
  return { text: "Good", color: "text-success" };
}

export function AudioSettings({ className = "" }: AudioSettingsProps) {
  const { settings, updateAudioDevice } = useSettings();
  const { devices, isLoading, error, refetch } = useAudioDevices();
  const { toast } = useToast();

  // Monitor audio level for the selected device
  const { level, isMonitoring } = useAudioLevelMonitor({
    deviceName: settings.audio.selectedDevice,
    enabled: !isLoading && !error,
  });

  // Wake word sensitivity (local state - would connect to backend in future)
  const [sensitivity, setSensitivity] = useState<Sensitivity>("medium");

  const handleDeviceChange = async (value: string) => {
    const deviceName = value === "system-default" ? null : value;
    await updateAudioDevice(deviceName);
    toast({
      type: "success",
      title: "Setting saved",
      description: deviceName
        ? `Audio input changed to ${deviceName}.`
        : "Using system default audio input.",
    });
  };

  const handleRefresh = () => {
    refetch();
    toast({
      type: "info",
      title: "Refreshing devices",
      description: "Scanning for audio input devices...",
    });
  };

  const handleSensitivityChange = (value: string) => {
    setSensitivity(value as Sensitivity);
    toast({
      type: "success",
      title: "Setting saved",
      description: `Wake word sensitivity set to ${value}.`,
    });
  };

  const levelIndicator = getLevelIndicator(level);

  // Check if selected device is currently available
  const selectedDevice = settings.audio.selectedDevice;
  const isSelectedDeviceAvailable =
    selectedDevice === null || devices.some((d) => d.name === selectedDevice);

  return (
    <div className={`space-y-6 ${className}`.trim()}>
      {/* Audio Input Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Audio Input
        </h2>
        <Card>
          <CardContent className="space-y-4">
            {/* Input Device */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label
                  htmlFor="audio-device-select"
                  className="text-sm font-medium text-text-primary"
                >
                  Input Device
                </label>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleRefresh}
                  disabled={isLoading}
                  aria-label="Refresh device list"
                >
                  <RefreshCw
                    className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`}
                  />
                  Refresh
                </Button>
              </div>

              {error ? (
                <div className="text-sm text-error">
                  Failed to load devices.{" "}
                  <button
                    type="button"
                    className="underline hover:no-underline"
                    onClick={refetch}
                  >
                    Retry
                  </button>
                </div>
              ) : isLoading ? (
                <div className="text-sm text-text-secondary">
                  Loading devices...
                </div>
              ) : (
                <Select
                  value={selectedDevice ?? "system-default"}
                  onValueChange={handleDeviceChange}
                  placeholder="Select audio device"
                >
                  <SelectItem value="system-default">System Default</SelectItem>
                  {devices.map((device) => (
                    <SelectItem key={device.name} value={device.name}>
                      {device.name}
                      {device.isDefault ? " (Default)" : ""}
                    </SelectItem>
                  ))}
                </Select>
              )}

              {!isSelectedDeviceAvailable && selectedDevice && (
                <div className="text-sm text-warning">
                  Selected device "{selectedDevice}" is not connected. Recording
                  will use system default.
                </div>
              )}
            </div>

            {/* Audio Level Meter */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-text-secondary">Audio Level</span>
                <span className={`text-sm font-medium ${levelIndicator.color}`}>
                  {isMonitoring ? levelIndicator.text : "Not monitoring"}
                </span>
              </div>
              <AudioLevelMeter level={level} />
              <p className="text-xs text-text-secondary">
                Test your microphone input
              </p>
            </div>
          </CardContent>
        </Card>
      </section>

      {/* Wake Word Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Wake Word
        </h2>
        <Card>
          <CardContent className="space-y-4">
            {/* Wake Phrase Display */}
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-text-primary">
                Wake Phrase
              </span>
              <span className="text-sm text-text-secondary font-mono">
                "Hey Cat"
              </span>
            </div>

            {/* Sensitivity Slider - Using Select as simple alternative */}
            <div className="space-y-2">
              <label
                htmlFor="sensitivity-select"
                className="text-sm font-medium text-text-primary"
              >
                Sensitivity
              </label>
              <Select
                value={sensitivity}
                onValueChange={handleSensitivityChange}
              >
                <SelectItem value="low">Low</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="high">High</SelectItem>
              </Select>
              <p className="text-xs text-text-secondary">
                {sensitivity === "low" &&
                  "Less sensitive - requires clearer pronunciation"}
                {sensitivity === "medium" &&
                  "Balanced sensitivity for most environments"}
                {sensitivity === "high" &&
                  "More sensitive - may trigger more easily in noisy environments"}
              </p>
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
