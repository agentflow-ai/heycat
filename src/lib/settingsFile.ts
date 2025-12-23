import { invoke } from "@tauri-apps/api/core";

/**
 * Cached settings file name for the current worktree context.
 * - Returns `settings-{identifier}.json` when running in a worktree
 * - Returns `settings.json` when running in main repository
 *
 * This value is fetched once at app startup and cached for the session.
 */
let cachedSettingsFile: string | null = null;

/**
 * Get the settings file name for the current worktree context.
 * Fetches from backend on first call, then returns cached value.
 *
 * This enables worktree-specific settings isolation so multiple heycat
 * instances can run with different configurations (e.g., different hotkeys).
 */
export async function getSettingsFile(): Promise<string> {
  if (cachedSettingsFile !== null) {
    return cachedSettingsFile;
  }

  try {
    cachedSettingsFile = await invoke<string>("get_settings_file_name");
    return cachedSettingsFile;
  } catch {
    // Fallback to default if backend call fails
    cachedSettingsFile = "settings.json";
    return cachedSettingsFile;
  }
}

/**
 * Initialize the settings file name at app startup.
 * Called from AppInitializer to ensure the value is ready before use.
 */
export async function initializeSettingsFile(): Promise<void> {
  await getSettingsFile();
}
