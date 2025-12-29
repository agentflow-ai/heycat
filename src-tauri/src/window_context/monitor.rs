// Background window monitoring for context detection
//
// Polls the active window at regular intervals and emits events
// when the window or matched context changes.

use super::types::ActiveWindowInfo;
use super::{get_active_window, WindowContext};
use crate::events::window_context_events::{self, ActiveWindowChangedPayload};
use crate::spacetimedb::client::SpacetimeClient;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// Find the highest-priority matching context for a window from a list of contexts
fn find_matching_context<'a>(
    contexts: &'a [WindowContext],
    window: &ActiveWindowInfo,
) -> Option<&'a WindowContext> {
    contexts
        .iter()
        .filter(|ctx| ctx.enabled)
        .filter(|ctx| {
            // Case-insensitive app name match
            ctx.matcher.app_name.to_lowercase() == window.app_name.to_lowercase()
        })
        .filter(|ctx| {
            // Check title pattern if present
            match (&ctx.matcher.title_pattern, &window.window_title) {
                (Some(pattern), Some(title)) => {
                    // Try to compile and match regex
                    Regex::new(pattern)
                        .map(|re| re.is_match(title))
                        .unwrap_or(false)
                }
                (Some(_), None) => false,
                (None, _) => true,
            }
        })
        .max_by_key(|ctx| ctx.priority)
}

/// Default polling interval in milliseconds
const DEFAULT_POLL_INTERVAL_MS: u64 = 200;

/// Configuration for the window monitor
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Polling interval in milliseconds
    pub poll_interval_ms: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
        }
    }
}

/// Background monitor that tracks the active window and matches contexts
pub struct WindowMonitor {
    /// Flag to signal the thread to stop
    running: Arc<AtomicBool>,
    /// Currently matched context ID
    current_context: Arc<Mutex<Option<Uuid>>>,
    /// Handle to the background thread
    thread_handle: Option<JoinHandle<()>>,
    /// Monitor configuration
    config: MonitorConfig,
}

impl WindowMonitor {
    /// Create a new window monitor with default configuration
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            current_context: Arc::new(Mutex::new(None)),
            thread_handle: None,
            config: MonitorConfig::default(),
        }
    }

    /// Create a new window monitor with custom configuration
    pub fn with_config(config: MonitorConfig) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            current_context: Arc::new(Mutex::new(None)),
            thread_handle: None,
            config,
        }
    }

    /// Start the background monitoring thread
    ///
    /// # Arguments
    /// * `app_handle` - Tauri app handle for event emission
    /// * `spacetime_client` - SpacetimeDB client for window context retrieval
    ///
    /// # Returns
    /// Ok(()) if started successfully, Err if already running
    pub fn start(
        &mut self,
        app_handle: AppHandle,
        spacetime_client: Arc<Mutex<SpacetimeClient>>,
    ) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Monitor is already running".to_string());
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let current_context = self.current_context.clone();
        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);

        let handle = thread::spawn(move || {
            crate::info!("[WindowMonitor] Starting background monitoring");

            let mut last_window: Option<ActiveWindowInfo> = None;
            let mut last_context_id: Option<Uuid> = None;

            while running.load(Ordering::SeqCst) {
                // Get current active window
                match get_active_window() {
                    Ok(window) => {
                        // Check if window changed
                        let window_changed = match &last_window {
                            Some(last) => {
                                last.app_name != window.app_name
                                    || last.window_title != window.window_title
                                    || last.pid != window.pid
                            }
                            None => true,
                        };

                        if window_changed {
                            // Find matching context from SpacetimeDB
                            let matched_context = spacetime_client
                                .lock()
                                .ok()
                                .and_then(|client| {
                                    client.list_window_contexts().ok()
                                })
                                .and_then(|contexts| {
                                    find_matching_context(&contexts, &window)
                                        .map(|ctx| (ctx.id, ctx.name.clone()))
                                });

                            let matched_context_id = matched_context.as_ref().map(|(id, _)| *id);
                            let matched_context_name = matched_context.map(|(_, name)| name);

                            // Update current context
                            if let Ok(mut guard) = current_context.lock() {
                                *guard = matched_context_id;
                            }

                            // Emit event
                            let payload = ActiveWindowChangedPayload {
                                app_name: window.app_name.clone(),
                                bundle_id: window.bundle_id.clone(),
                                window_title: window.window_title.clone(),
                                matched_context_id: matched_context_id.map(|id| id.to_string()),
                                matched_context_name,
                            };

                            if let Err(e) = app_handle.emit(
                                window_context_events::ACTIVE_WINDOW_CHANGED,
                                payload,
                            ) {
                                crate::warn!(
                                    "[WindowMonitor] Failed to emit active_window_changed: {}",
                                    e
                                );
                            }

                            // Log context change if it changed
                            if matched_context_id != last_context_id {
                                match matched_context_id {
                                    Some(id) => {
                                        crate::debug!(
                                            "[WindowMonitor] Context matched: {} ({})",
                                            id,
                                            window.app_name
                                        );
                                    }
                                    None if last_context_id.is_some() => {
                                        crate::debug!(
                                            "[WindowMonitor] Context cleared ({})",
                                            window.app_name
                                        );
                                    }
                                    None => {}
                                }
                                last_context_id = matched_context_id;
                            }

                            last_window = Some(window);
                        }
                    }
                    Err(e) => {
                        // Log warning but continue - window detection can temporarily fail
                        crate::warn!("[WindowMonitor] Failed to get active window: {}", e);
                    }
                }

                thread::sleep(poll_interval);
            }

            crate::info!("[WindowMonitor] Background monitoring stopped");
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop the background monitoring thread
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| "Failed to join monitor thread".to_string())?;
        }

        // Clear current context
        if let Ok(mut guard) = self.current_context.lock() {
            *guard = None;
        }

        Ok(())
    }

    /// Get the currently matched context ID
    pub fn get_current_context(&self) -> Option<Uuid> {
        self.current_context.lock().ok().and_then(|guard| *guard)
    }

    /// Check if the monitor is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for WindowMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WindowMonitor {
    fn drop(&mut self) {
        if self.is_running() {
            if let Err(e) = self.stop() {
                crate::warn!("[WindowMonitor] Error stopping monitor on drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
#[path = "monitor_test.rs"]
mod tests;
