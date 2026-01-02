// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Enable coverage attribute on nightly for explicit exclusions
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod activation;
mod app;
mod audio;
mod audio_constants;
mod commands;
mod dictionary;
mod events;
mod hotkey;
mod keyboard;
mod keyboard_capture;
mod model;
mod parakeet;
mod paths;
mod recording;
mod shutdown;
mod storage;
mod swift;
mod transcription;
mod turso;
mod util;
mod voice_commands;
mod wake_handler;
mod window_context;
mod worktree;

#[cfg(test)]
pub mod test_utils;

use tauri::WindowEvent;
use tauri_plugin_log::{Target, TargetKind};

// Re-export log macros for use throughout the crate
pub use tauri_plugin_log::log::{debug, error, info, trace, warn};

/// Application entry point - starts the Tauri event loop.
/// Note: This function cannot be unit tested as it starts a GUI.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("heycat".to_string()),
                    }),
                ])
                .level(if cfg!(debug_assertions) {
                    tauri_plugin_log::log::LevelFilter::Debug
                } else {
                    tauri_plugin_log::log::LevelFilter::Info
                })
                // Suppress verbose DEBUG logs from tract ONNX inference library
                .level_for("tract_core", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_onnx", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_hir", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_linalg", tauri_plugin_log::log::LevelFilter::Warn)
                // Suppress verbose INFO logs from ONNX Runtime during model loading
                .level_for("ort", tauri_plugin_log::log::LevelFilter::Warn)
                .build(),
        )
        .setup(|app| app::setup(app))
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                app::on_window_destroyed(window);
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Recording commands
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::get_recording_state,
            commands::recording::get_last_recording_buffer,
            commands::recording::clear_last_recording_buffer,
            commands::recording::list_recordings,
            commands::recording::delete_recording,
            // Transcription commands
            commands::transcription::transcribe_file,
            commands::transcription::list_transcriptions,
            commands::transcription::get_transcriptions_by_recording,
            // Audio commands
            commands::audio::list_audio_devices,
            commands::audio::start_audio_monitor,
            commands::audio::stop_audio_monitor,
            commands::audio::init_audio_monitor,
            // Model commands
            model::check_parakeet_model_status,
            model::download_model,
            // Voice commands
            voice_commands::get_commands,
            voice_commands::add_command,
            voice_commands::update_command,
            voice_commands::remove_command,
            voice_commands::executor::test_command,
            // Hotkey commands
            commands::hotkey::suspend_recording_shortcut,
            commands::hotkey::resume_recording_shortcut,
            commands::hotkey::update_recording_shortcut,
            commands::hotkey::get_recording_shortcut,
            commands::hotkey::get_recording_mode,
            commands::hotkey::set_recording_mode,
            commands::hotkey::start_shortcut_recording,
            commands::hotkey::stop_shortcut_recording,
            commands::hotkey::open_accessibility_preferences,
            // Worktree commands
            commands::get_settings_file_name,
            // Dictionary commands
            commands::dictionary::list_dictionary_entries,
            commands::dictionary::add_dictionary_entry,
            commands::dictionary::update_dictionary_entry,
            commands::dictionary::delete_dictionary_entry,
            // Window context commands
            commands::window_context::get_active_window_info,
            commands::window_context::list_running_applications,
            commands::window_context::list_window_contexts,
            commands::window_context::add_window_context,
            commands::window_context::update_window_context,
            commands::window_context::delete_window_context,
            // Window commands
            commands::window::show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
