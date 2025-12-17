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
