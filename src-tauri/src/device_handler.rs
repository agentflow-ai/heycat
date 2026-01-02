//! Audio device change handler for macOS.
//!
//! This module handles audio device connection/disconnection notifications from
//! Core Audio and coordinates restarting the audio engine to ensure fresh
//! hardware connection after device changes.

use std::sync::OnceLock;

/// Static storage for the device change handler state.
/// Uses OnceLock for safe, one-time initialization.
static DEVICE_HANDLER: OnceLock<DeviceHandlerState> = OnceLock::new();

/// State required for handling device change events.
struct DeviceHandlerState {
    /// Marker to indicate handler is initialized (no actual state needed)
    _initialized: bool,
}

// Ensure DeviceHandlerState is Send + Sync for static storage
unsafe impl Send for DeviceHandlerState {}
unsafe impl Sync for DeviceHandlerState {}

/// Initialize the device change handler.
///
/// This should be called once during app setup, typically in lib.rs.
/// The handler will restart the audio engine when devices connect/disconnect.
pub fn init_device_change_handler() {
    let state = DeviceHandlerState { _initialized: true };

    if DEVICE_HANDLER.set(state).is_err() {
        crate::warn!("Device change handler already initialized");
        return;
    }

    // Register the Swift callback for device change notifications
    crate::swift::register_device_change_callback(on_device_change);
    crate::info!("Device change handler initialized - listening for device changes");
}

/// Callback invoked when audio devices connect or disconnect.
///
/// This is called from the Swift layer via FFI. It spawns an async task
/// to restart the audio engine without blocking the callback.
extern "C" fn on_device_change() {
    crate::info!("Audio device change detected - scheduling audio engine restart");

    // Verify handler is initialized
    if DEVICE_HANDLER.get().is_none() {
        crate::error!("Device change handler not initialized - cannot restart audio engine");
        return;
    }

    // Spawn async task to restart the audio engine
    // We use tauri's async runtime to avoid blocking the callback
    tauri::async_runtime::spawn(async move {
        restart_audio_engine_async().await;
    });
}

/// Async task that restarts the audio engine if it was running.
///
/// This ensures a fresh hardware connection after device changes.
/// The sequence:
/// 1. Check if audio engine is running
/// 2. Stop the audio engine
/// 3. Wait 200ms for Core Audio cleanup
/// 4. Start the audio engine with default device
async fn restart_audio_engine_async() {
    // Check if audio engine is running
    let audio_was_running =
        tauri::async_runtime::spawn_blocking(crate::swift::audio_engine_is_running)
            .await
            .unwrap_or(false);

    if !audio_was_running {
        crate::debug!("Audio engine was not running - no restart needed");
        return;
    }

    crate::info!("Audio engine was running - restarting for fresh hardware connection");

    // Stop the audio engine
    if let Err(e) = tauri::async_runtime::spawn_blocking(crate::swift::audio_engine_stop).await {
        crate::warn!("Failed to stop audio engine: {}", e);
    }

    // Wait 200ms for Core Audio cleanup
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Start the audio engine with default device
    let start_result =
        tauri::async_runtime::spawn_blocking(|| crate::swift::audio_engine_start(None)).await;

    match start_result {
        Ok(crate::swift::AudioEngineResult::Ok) => {
            crate::info!("Audio engine restarted successfully after device change");
        }
        Ok(crate::swift::AudioEngineResult::Failed(e)) => {
            crate::error!("Failed to restart audio engine after device change: {}", e);
        }
        Err(e) => {
            crate::error!("Audio engine restart task panicked: {}", e);
        }
    }
}

#[cfg(test)]
#[path = "device_handler_test.rs"]
mod tests;
