// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Enable coverage attribute on nightly for explicit exclusions
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod audio;
mod commands;
mod events;
mod hotkey;
mod recording;

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Concrete type for HotkeyService with TauriShortcutBackend
type HotkeyServiceHandle = hotkey::HotkeyService<hotkey::TauriShortcutBackend>;

/// Greets the user with a personalized message.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Application entry point - starts the Tauri event loop.
/// Note: This function cannot be unit tested as it starts a GUI.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            eprintln!("[app] Setting up heycat...");

            // Create shared state for recording manager
            let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));

            // Manage the state for Tauri commands
            app.manage(recording_state.clone());

            // Create event emitter, audio thread, and hotkey integration
            eprintln!("[app] Creating audio thread...");
            let emitter = commands::TauriEventEmitter::new(app.handle().clone());
            let audio_thread = Arc::new(audio::AudioThreadHandle::spawn());
            eprintln!("[app] Audio thread spawned");

            // Manage audio thread state for Tauri commands
            app.manage(audio_thread.clone());

            let integration = Arc::new(Mutex::new(
                hotkey::HotkeyIntegration::new(emitter).with_audio_thread(audio_thread),
            ));

            // Clone for callback
            let integration_clone = integration.clone();
            let state_clone = recording_state.clone();

            // Register hotkey
            eprintln!("[app] Registering global hotkey (Cmd+Shift+R)...");
            let backend = hotkey::TauriShortcutBackend::new(app.handle().clone());
            let service = hotkey::HotkeyService::new(backend);

            service
                .register_recording_shortcut(Box::new(move || {
                    eprintln!("[app] Hotkey pressed!");
                    match integration_clone.lock() {
                        Ok(mut guard) => {
                            guard.handle_toggle(&state_clone);
                        }
                        Err(e) => {
                            eprintln!("[app] Failed to acquire integration lock: {}", e);
                        }
                    }
                }))
                .expect("Failed to register recording hotkey");

            // Store service in state for cleanup on exit
            app.manage(service);

            eprintln!("[app] Setup complete! Ready to record.");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                eprintln!("[app] Window destroyed, cleaning up...");
                // Unregister hotkey on window close
                if let Some(service) = window.app_handle().try_state::<HotkeyServiceHandle>() {
                    if let Err(e) = service.unregister_recording_shortcut() {
                        eprintln!("[app] Failed to unregister hotkey: {:?}", e);
                    } else {
                        eprintln!("[app] Hotkey unregistered successfully");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_state,
            commands::get_last_recording_buffer,
            commands::clear_last_recording_buffer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_with_name() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_greet_with_empty_name() {
        let result = greet("");
        assert_eq!(result, "Hello, ! You've been greeted from Rust!");
    }

    #[test]
    fn test_greet_with_special_characters() {
        let result = greet("Test<User>");
        assert_eq!(result, "Hello, Test<User>! You've been greeted from Rust!");
    }
}
