/**
 * Represents an audio input device returned from the backend
 */
export interface AudioInputDevice {
  name: string;
  isDefault: boolean;
}

/**
 * Audio-related settings stored in the frontend
 */
export interface AudioSettings {
  /** Name of the selected device, or null for system default */
  selectedDevice: string | null;
}

/**
 * Default audio settings for fresh installs
 */
export const DEFAULT_AUDIO_SETTINGS: AudioSettings = {
  selectedDevice: null,
};

/**
 * Audio device error types matching the backend AudioDeviceError enum
 * These are emitted via the 'audio_device_error' event
 */
export type AudioDeviceErrorType =
  | "deviceNotFound"
  | "noDevicesAvailable"
  | "deviceDisconnected"
  | "captureError";

/**
 * Discriminated union for audio device errors
 * The 'type' field determines which additional fields are present
 */
export type AudioDeviceError =
  | { type: "deviceNotFound"; deviceName: string }
  | { type: "noDevicesAvailable" }
  | { type: "deviceDisconnected" }
  | { type: "captureError"; message: string };

/**
 * Extract the error type from an AudioDeviceError
 */
export function getErrorType(error: AudioDeviceError): AudioDeviceErrorType {
  return error.type;
}

/**
 * Get a human-readable message for an audio device error
 */
export function getErrorMessage(error: AudioDeviceError): string {
  switch (error.type) {
    case "deviceNotFound":
      return `The selected microphone "${error.deviceName}" is not connected.`;
    case "noDevicesAvailable":
      return "No audio input devices were found. Please connect a microphone.";
    case "deviceDisconnected":
      return "The microphone was disconnected during recording.";
    case "captureError":
      return error.message || "An error occurred while recording.";
  }
}
