//! Window management commands.

use tauri::{AppHandle, Manager};

/// Show the main window, close the splash window, and give main focus
///
/// Called by the frontend when the app is ready to be displayed (e.g., after
/// initialization completes). This enables a seamless splash-to-app transition.
///
/// Includes error recovery with retry logic for splash window operations.
#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    // Show the main window first (before closing splash) for smoother UX
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))?;

    crate::info!("Main window shown and focused");

    // Close the splash window with retry logic
    if let Some(splash) = app_handle.get_webview_window("splash") {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        const RETRY_DELAY_MS: u64 = 50;

        loop {
            attempts += 1;
            match splash.close() {
                Ok(()) => {
                    crate::debug!("Splash window closed");
                    break;
                }
                Err(e) => {
                    if attempts >= MAX_ATTEMPTS {
                        // Log warning but don't fail - main window is already visible
                        crate::warn!(
                            "Failed to close splash window after {} attempts: {}",
                            attempts,
                            e
                        );
                        break;
                    }
                    crate::debug!(
                        "Splash close attempt {} failed, retrying: {}",
                        attempts,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                }
            }
        }
    }

    Ok(())
}
