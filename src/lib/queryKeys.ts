/**
 * Centralized query key definitions for Tanstack Query.
 *
 * Convention: ['tauri', 'command_name', ...args]
 * - Prefix 'tauri' namespaces all Tauri commands
 * - Command names match the Rust #[tauri::command] function names
 * - Additional args for parameterized queries
 *
 * Using `as const` ensures the arrays are readonly tuples with literal types,
 * enabling proper type inference in query hooks.
 */
export const queryKeys = {
  tauri: {
    /** Query key for list_recordings command */
    listRecordings: ["tauri", "list_recordings"] as const,

    /** Query key for get_recording_state command */
    getRecordingState: ["tauri", "get_recording_state"] as const,

    /** Query key for list_audio_devices command */
    listAudioDevices: ["tauri", "list_audio_devices"] as const,

    /** Query key for get_listening_status command */
    getListeningStatus: ["tauri", "get_listening_status"] as const,

    /** Query key factory for check_parakeet_model_status command */
    checkModelStatus: (type: string) =>
      ["tauri", "check_parakeet_model_status", type] as const,
  },
  dictionary: {
    /** Base key for all dictionary queries */
    all: ["dictionary"] as const,
    /** Query key for list_dictionary_entries command */
    list: () => [...queryKeys.dictionary.all, "list"] as const,
  },
  windowContext: {
    /** Base key for all window context queries */
    all: ["windowContext"] as const,
    /** Query key for list_window_contexts command */
    list: () => [...queryKeys.windowContext.all, "list"] as const,
    /** Query key for list_running_applications command */
    runningApps: () => [...queryKeys.windowContext.all, "runningApps"] as const,
  },
} as const;

/** Type for the query keys object */
export type QueryKeys = typeof queryKeys;

/** Type for extracting query key arrays */
export type TauriQueryKey =
  | typeof queryKeys.tauri.listRecordings
  | typeof queryKeys.tauri.getRecordingState
  | typeof queryKeys.tauri.listAudioDevices
  | typeof queryKeys.tauri.getListeningStatus
  | ReturnType<typeof queryKeys.tauri.checkModelStatus>;
