//! Audio device change handler for macOS.
//!
//! This module handles audio device connection/disconnection notifications from
//! Core Audio and coordinates restarting the audio engine to ensure fresh
//! hardware connection after device changes.
//!
//! Includes coordination with user-initiated device changes to prevent race
//! conditions when both the user and automatic handler try to control the
//! audio engine simultaneously.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex as TokioMutex;

/// Static storage for the device change handler state.
/// Uses OnceLock for safe, one-time initialization.
static DEVICE_HANDLER: OnceLock<DeviceHandlerState> = OnceLock::new();

/// Timestamp of last user-initiated device change (millis since UNIX_EPOCH).
/// Used to suppress automatic restarts during user device switching.
static LAST_USER_DEVICE_CHANGE: AtomicU64 = AtomicU64::new(0);

/// Duration to suppress auto-restart after user-initiated device change.
/// Extended to 1000ms to account for slower device switches and Core Audio propagation.
const SUPPRESSION_WINDOW_MS: u64 = 1000;

/// Delay before executing auto-restart to debounce rapid device changes.
const DEBOUNCE_DELAY_MS: u64 = 300;

/// Debounce state for auto-restart - holds pending restart task handle.
static RESTART_DEBOUNCE: OnceLock<TokioMutex<Option<tokio::task::JoinHandle<()>>>> = OnceLock::new();

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

/// Get the debounce mutex, initializing if needed.
fn get_debounce() -> &'static TokioMutex<Option<tokio::task::JoinHandle<()>>> {
    RESTART_DEBOUNCE.get_or_init(|| TokioMutex::new(None))
}

/// Mark that a user-initiated device change is occurring.
///
/// Call this BEFORE calling `audio_engine_set_device()` to suppress
/// automatic restart callbacks that would otherwise race with the user action.
pub fn mark_user_device_change() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    LAST_USER_DEVICE_CHANGE.store(now, Ordering::SeqCst);
    crate::debug!("Marked user-initiated device change at {}", now);
}

/// Check if we should suppress auto-restart due to recent user action.
fn should_suppress_auto_restart() -> bool {
    let last_change = LAST_USER_DEVICE_CHANGE.load(Ordering::SeqCst);
    if last_change == 0 {
        return false;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let elapsed = now.saturating_sub(last_change);
    elapsed < SUPPRESSION_WINDOW_MS
}

/// Callback invoked when audio devices connect or disconnect.
///
/// This is called from the Swift layer via FFI. It uses suppression and
/// debounce to coordinate with user-initiated device changes:
/// - If a user device change occurred within SUPPRESSION_WINDOW_MS, skip auto-restart
/// - Debounce rapid device changes (USB flapping) by canceling pending restarts
extern "C" fn on_device_change() {
    crate::info!("Audio device change detected");

    // Verify handler is initialized
    if DEVICE_HANDLER.get().is_none() {
        crate::error!("Device change handler not initialized - cannot restart audio engine");
        return;
    }

    // Check if this should be suppressed (user-initiated change in progress)
    if should_suppress_auto_restart() {
        crate::info!("Auto-restart suppressed - user-initiated device change in progress");
        return;
    }

    // Spawn async task with debounce for rapid device flapping
    tauri::async_runtime::spawn(async move {
        let mut guard = get_debounce().lock().await;

        // Cancel any pending restart
        if let Some(handle) = guard.take() {
            handle.abort();
            crate::debug!("Cancelled pending auto-restart (debounce)");
        }

        // Schedule new restart after debounce delay
        // Use tokio::spawn directly to get a tokio::task::JoinHandle
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;

            // Re-check suppression after debounce (user may have initiated change)
            if should_suppress_auto_restart() {
                crate::info!("Auto-restart suppressed after debounce");
                return;
            }

            restart_audio_engine_async().await;
        });

        *guard = Some(handle);
    });
}

/// Async task that restarts the audio engine if it was running.
///
/// This ensures a fresh hardware connection after device changes.
/// The sequence:
/// 1. Check if audio capture is in progress (skip if so - don't interrupt recording)
/// 2. Check if audio engine is running
/// 3. Stop the audio engine
/// 4. Wait 200ms for Core Audio cleanup
/// 5. Start the audio engine with default device
async fn restart_audio_engine_async() {
    // Check if audio CAPTURE is in progress - if so, skip auto-restart to preserve recording
    // The Swift side will handle device switching while preserving capture state
    let is_capturing =
        tauri::async_runtime::spawn_blocking(crate::swift::audio_engine_is_capturing)
            .await
            .unwrap_or(false);

    if is_capturing {
        crate::info!("Audio capture in progress - skipping auto-restart to preserve recording");
        return;
    }

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
