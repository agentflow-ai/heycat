// Shutdown coordination module
// Provides a global flag to prevent operations during app shutdown

use core_foundation::base::TCFType;
use core_foundation::runloop::{CFRunLoop, CFRunLoopStop};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::AppHandle;

/// Global shutdown flag - set to true when app is shutting down
static APP_SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Global reference to CGEventTap's run loop for graceful shutdown
static CGEVENTTAP_RUN_LOOP: Mutex<Option<CFRunLoop>> = Mutex::new(None);

/// Global reference to the Tauri app handle for graceful shutdown (e.g. SIGINT in dev)
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

/// Signal that the app is shutting down
/// Call this first in WindowEvent::Destroyed before any cleanup
pub fn signal_shutdown() {
    APP_SHUTTING_DOWN.store(true, Ordering::SeqCst);
    eprintln!("[PASTE-TRACE] signal_shutdown() called - flag is now TRUE");
    crate::info!("App shutdown signaled");
}

/// Check if the app is shutting down
/// Returns true after signal_shutdown() has been called
pub fn is_shutting_down() -> bool {
    APP_SHUTTING_DOWN.load(Ordering::SeqCst)
}

/// Register the Tauri app handle for graceful shutdown coordination.
///
/// This enables the Ctrl+C (SIGINT) handler to request a clean exit via `AppHandle::exit(0)`
/// instead of calling `std::process::exit(0)` (which skips destructors and can leave the
/// system in a bad state).
pub fn register_app_handle(handle: AppHandle) {
    if let Ok(mut guard) = APP_HANDLE.lock() {
        *guard = Some(handle);
    }
}

/// Request a graceful app exit (if an AppHandle has been registered).
pub fn request_app_exit(exit_code: i32) {
    // Clone out of the mutex so we don't hold the lock while calling into Tauri.
    let handle = APP_HANDLE.lock().ok().and_then(|g| g.as_ref().cloned());
    if let Some(handle) = handle {
        handle.exit(exit_code);
    }
}

/// Register the CGEventTap's run loop for graceful shutdown
/// Call this when starting the CGEventTap to enable clean termination
pub fn register_cgeventtap_run_loop(run_loop: CFRunLoop) {
    if let Ok(mut guard) = CGEVENTTAP_RUN_LOOP.lock() {
        *guard = Some(run_loop);
        crate::debug!("CGEventTap run loop registered for shutdown coordination");
    }
}

/// Stop the CGEventTap's run loop for graceful shutdown
/// Call this before exit to prevent spurious events during cleanup
pub fn stop_cgeventtap() {
    if let Ok(guard) = CGEVENTTAP_RUN_LOOP.lock() {
        if let Some(ref run_loop) = *guard {
            unsafe {
                CFRunLoopStop(run_loop.as_concrete_TypeRef());
            }
            crate::debug!("CGEventTap run loop stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_flag_transitions() {
        // Reset for test isolation (note: tests run in parallel, so this is imperfect)
        APP_SHUTTING_DOWN.store(false, Ordering::SeqCst);

        // Initially not shutting down
        assert!(!is_shutting_down());

        // After signal, should be shutting down
        signal_shutdown();
        assert!(is_shutting_down());

        // Should remain true
        assert!(is_shutting_down());
    }
}
