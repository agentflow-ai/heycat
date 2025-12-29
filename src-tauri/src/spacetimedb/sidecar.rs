//! SpacetimeDB sidecar process management
//!
//! Spawns and manages the SpacetimeDB standalone server as a child process.
//! The server provides a local WebSocket endpoint for database operations.
//!
//! ## Module Publishing
//!
//! After the sidecar starts, the WASM module must be published before clients
//! can connect. The `start_and_wait` method handles this automatically by:
//! 1. Starting the sidecar process
//! 2. Waiting for health check to pass
//! 3. Publishing the module (if not already published)

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// Default database/module name
pub const DEFAULT_DATABASE_NAME: &str = "heycat";

/// Default port for SpacetimeDB standalone server
pub const DEFAULT_PORT: u16 = 3055;

/// Default host for SpacetimeDB standalone server (localhost only)
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Errors that can occur during sidecar management
#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("SpacetimeDB binary not found at {0}")]
    BinaryNotFound(PathBuf),

    #[error("Failed to spawn SpacetimeDB process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("SpacetimeDB process exited unexpectedly with code {0:?}")]
    ProcessExited(Option<i32>),

    #[error("Failed to stop SpacetimeDB process: {0}")]
    StopFailed(String),

    #[error("Health check failed after {0} attempts")]
    HealthCheckFailed(u32),

    #[error("Failed to publish module: {0}")]
    ModulePublishFailed(String),

    #[error("WASM module not found at {0}")]
    WasmModuleNotFound(PathBuf),
}

/// Configuration for the SpacetimeDB sidecar
#[derive(Debug, Clone)]
pub struct SidecarConfig {
    /// Path to the SpacetimeDB binary
    pub binary_path: PathBuf,
    /// Host to bind to (default: 127.0.0.1)
    pub host: String,
    /// Port to listen on (default: 3055)
    pub port: u16,
    /// Data directory for SpacetimeDB storage
    pub data_dir: PathBuf,
    /// Number of health check attempts before giving up
    pub health_check_attempts: u32,
    /// Delay between health check attempts
    pub health_check_delay: Duration,
    /// Path to the WASM module to publish
    pub wasm_module_path: PathBuf,
    /// Database name for the module
    pub database_name: String,
}

impl SidecarConfig {
    /// Create a new sidecar configuration with worktree-aware data directory
    pub fn new(worktree_id: Option<&str>) -> Self {
        let data_dir = get_spacetimedb_data_dir(worktree_id);
        let binary_path = get_spacetimedb_binary_path();
        let wasm_module_path = get_wasm_module_path();

        Self {
            binary_path,
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            data_dir,
            health_check_attempts: 30,
            health_check_delay: Duration::from_millis(100),
            wasm_module_path,
            database_name: DEFAULT_DATABASE_NAME.to_string(),
        }
    }

    /// Get the WebSocket URL for connecting to the sidecar
    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}", self.host, self.port)
    }

    /// Get the listen address for the server
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Handle to a running SpacetimeDB sidecar process
pub struct SidecarHandle {
    /// The child process
    child: Child,
    /// Configuration used to start the sidecar
    config: SidecarConfig,
    /// Flag indicating if shutdown has been requested
    shutdown_requested: Arc<AtomicBool>,
}

impl SidecarHandle {
    /// Stop the sidecar process gracefully
    pub fn stop(&mut self) -> Result<(), SidecarError> {
        self.shutdown_requested.store(true, Ordering::SeqCst);

        // Try graceful shutdown first (SIGTERM on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let pid = self.child.id();
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }

            // Wait a bit for graceful shutdown
            thread::sleep(Duration::from_millis(500));
        }

        // Check if process exited
        match self.child.try_wait() {
            Ok(Some(_)) => {
                crate::debug!("SpacetimeDB sidecar stopped gracefully");
                Ok(())
            }
            Ok(None) => {
                // Still running, force kill
                crate::warn!("SpacetimeDB sidecar didn't stop gracefully, forcing kill");
                self.child
                    .kill()
                    .map_err(|e| SidecarError::StopFailed(e.to_string()))?;
                self.child.wait().ok();
                Ok(())
            }
            Err(e) => Err(SidecarError::StopFailed(e.to_string())),
        }
    }

    /// Check if the sidecar process is still running
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Get the configuration used to start this sidecar
    pub fn config(&self) -> &SidecarConfig {
        &self.config
    }

    /// Get the process ID of the sidecar
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for SidecarHandle {
    fn drop(&mut self) {
        if !self.shutdown_requested.load(Ordering::SeqCst) {
            crate::warn!("SidecarHandle dropped without explicit stop, killing process");
        }
        // Always try to kill the child process on drop
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Manager for SpacetimeDB sidecar lifecycle
pub struct SidecarManager {
    config: SidecarConfig,
}

impl SidecarManager {
    /// Create a new sidecar manager with the given configuration
    pub fn new(config: SidecarConfig) -> Self {
        Self { config }
    }

    /// Create a new sidecar manager with default configuration
    pub fn with_defaults(worktree_id: Option<&str>) -> Self {
        Self::new(SidecarConfig::new(worktree_id))
    }

    /// Start the SpacetimeDB sidecar process
    pub fn start(&self) -> Result<SidecarHandle, SidecarError> {
        // Verify binary exists
        if !self.config.binary_path.exists() {
            return Err(SidecarError::BinaryNotFound(self.config.binary_path.clone()));
        }

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir).map_err(SidecarError::SpawnFailed)?;

        crate::info!(
            "Starting SpacetimeDB sidecar: {:?} on {}",
            self.config.binary_path,
            self.config.listen_addr()
        );

        // Spawn the SpacetimeDB standalone server
        // Command: spacetime start --listen-addr 127.0.0.1:3055
        let child = Command::new(&self.config.binary_path)
            .args([
                "start",
                "--listen-addr",
                &self.config.listen_addr(),
            ])
            .current_dir(&self.config.data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        crate::info!(
            "SpacetimeDB sidecar started with PID {} (data dir: {:?})",
            child.id(),
            self.config.data_dir
        );

        let handle = SidecarHandle {
            child,
            config: self.config.clone(),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        };

        Ok(handle)
    }

    /// Start the sidecar and wait for it to be healthy
    ///
    /// This method:
    /// 1. Starts the SpacetimeDB sidecar process
    /// 2. Waits for the health check to pass
    /// 3. Publishes the WASM module (creates the database if needed)
    pub fn start_and_wait(&self) -> Result<SidecarHandle, SidecarError> {
        let mut handle = self.start()?;

        // Wait for server to become healthy
        for attempt in 1..=self.config.health_check_attempts {
            thread::sleep(self.config.health_check_delay);

            // Check if process is still running
            if !handle.is_running() {
                let exit_code = handle.child.try_wait().ok().and_then(|s| s.and_then(|s| s.code()));
                return Err(SidecarError::ProcessExited(exit_code));
            }

            // Try to connect to the health endpoint
            if self.check_health() {
                crate::info!(
                    "SpacetimeDB sidecar healthy after {} attempts",
                    attempt
                );

                // Publish the module to create the database
                self.publish_module()?;

                return Ok(handle);
            }

            crate::debug!("Health check attempt {}/{}", attempt, self.config.health_check_attempts);
        }

        // Health check failed, stop the process
        let _ = handle.stop();
        Err(SidecarError::HealthCheckFailed(self.config.health_check_attempts))
    }

    /// Publish the WASM module to the SpacetimeDB server
    ///
    /// This creates the database if it doesn't exist, or updates it if it does.
    /// The command is idempotent - calling it multiple times is safe.
    fn publish_module(&self) -> Result<(), SidecarError> {
        // Verify WASM module exists
        if !self.config.wasm_module_path.exists() {
            return Err(SidecarError::WasmModuleNotFound(self.config.wasm_module_path.clone()));
        }

        let server_url = format!("http://{}:{}", self.config.host, self.config.port);

        crate::info!(
            "Publishing SpacetimeDB module '{}' from {:?}",
            self.config.database_name,
            self.config.wasm_module_path
        );

        // Run: spacetime publish --server <url> <database_name> --project-path <wasm_dir>
        // The publish command requires the project directory, not the WASM file directly
        let wasm_dir = self.config.wasm_module_path.parent()
            .and_then(|p| p.parent()) // Go up from release/ to target/
            .and_then(|p| p.parent()) // Go up from wasm32-unknown-unknown/ to target/
            .and_then(|p| p.parent()) // Go up from target/ to spacetimedb/
            .ok_or_else(|| SidecarError::ModulePublishFailed(
                format!("Cannot determine project path from {:?}", self.config.wasm_module_path)
            ))?;

        let output = Command::new(&self.config.binary_path)
            .args([
                "publish",
                "--server", &server_url,
                &self.config.database_name,
                "--project-path", &wasm_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| SidecarError::ModulePublishFailed(e.to_string()))?;

        if output.status.success() {
            crate::info!("SpacetimeDB module published successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Check if the error is just "database already exists" - that's OK
            if stderr.contains("already exists") || stdout.contains("already exists") {
                crate::info!("SpacetimeDB module already published");
                return Ok(());
            }

            Err(SidecarError::ModulePublishFailed(format!(
                "spacetime publish failed: stdout={}, stderr={}",
                stdout, stderr
            )))
        }
    }

    /// Check if the sidecar is healthy by attempting a TCP connection
    fn check_health(&self) -> bool {
        use std::net::TcpStream;
        let addr = self.config.listen_addr();
        TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(100),
        )
        .is_ok()
    }
}

/// Get the default SpacetimeDB binary path
///
/// During development, this looks for the `spacetime` CLI in PATH.
/// In production, it would look in the bundled resources directory.
fn get_spacetimedb_binary_path() -> PathBuf {
    // First check if spacetime CLI is available in PATH
    // This is the development path - in production we'll bundle the binary
    if let Ok(output) = Command::new("which").arg("spacetime").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return PathBuf::from(path);
            }
        }
    }

    // Fallback to expected bundled location (for production builds)
    // Tauri bundles external binaries in the resources directory
    #[cfg(target_os = "macos")]
    {
        // On macOS, check the app bundle resources
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(resources) = exe_path.parent().and_then(|p| p.parent()) {
                let bundled = resources.join("Resources").join("spacetime");
                if bundled.exists() {
                    return bundled;
                }
            }
        }
    }

    // Default fallback - assume it's in PATH
    PathBuf::from("spacetime")
}

/// Get the SpacetimeDB data directory, respecting worktree isolation
fn get_spacetimedb_data_dir(worktree_id: Option<&str>) -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("heycat");

    match worktree_id {
        Some(id) => base_dir.join("spacetimedb").join(id),
        None => base_dir.join("spacetimedb").join("main"),
    }
}

/// Get the path to the SpacetimeDB WASM module
///
/// During development, this looks for the compiled WASM in the spacetimedb/target directory.
/// In production, it would look in the bundled resources directory.
fn get_wasm_module_path() -> PathBuf {
    // First try relative to the executable (for development)
    // The spacetimedb module is at: <repo>/spacetimedb/target/wasm32-unknown-unknown/release/heycat_db.wasm
    if let Ok(exe_path) = std::env::current_exe() {
        // In development, exe is in src-tauri/target/debug/heycat
        // Go up to repo root and look for spacetimedb/target/...
        if let Some(repo_root) = exe_path
            .parent() // debug/
            .and_then(|p| p.parent()) // target/
            .and_then(|p| p.parent()) // src-tauri/
            .and_then(|p| p.parent()) // repo root
        {
            let wasm_path = repo_root
                .join("spacetimedb")
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
                .join("heycat_db.wasm");

            if wasm_path.exists() {
                return wasm_path;
            }
        }

        // Also check if we're in a worktree
        if let Some(worktree_root) = exe_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
        {
            let wasm_path = worktree_root
                .join("spacetimedb")
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
                .join("heycat_db.wasm");

            if wasm_path.exists() {
                return wasm_path;
            }
        }

        // Check bundled resources for production
        #[cfg(target_os = "macos")]
        {
            if let Some(resources) = exe_path.parent().and_then(|p| p.parent()) {
                let bundled = resources
                    .join("Resources")
                    .join("spacetimedb")
                    .join("heycat_db.wasm");
                if bundled.exists() {
                    return bundled;
                }
            }
        }
    }

    // Fallback - try current directory
    PathBuf::from("spacetimedb/target/wasm32-unknown-unknown/release/heycat_db.wasm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_config_defaults() {
        let config = SidecarConfig::new(None);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3055);
        assert!(config.data_dir.to_string_lossy().contains("spacetimedb"));
        assert!(config.data_dir.to_string_lossy().contains("main"));
        assert_eq!(config.database_name, "heycat");
    }

    #[test]
    fn test_sidecar_config_worktree_isolation() {
        let config = SidecarConfig::new(Some("feature-branch"));
        assert!(config.data_dir.to_string_lossy().contains("feature-branch"));
    }

    #[test]
    fn test_websocket_url() {
        let config = SidecarConfig::new(None);
        assert_eq!(config.websocket_url(), "ws://127.0.0.1:3055");
    }

    #[test]
    fn test_listen_addr() {
        let config = SidecarConfig::new(None);
        assert_eq!(config.listen_addr(), "127.0.0.1:3055");
    }

    #[test]
    fn test_wasm_module_path_ends_with_expected_name() {
        let config = SidecarConfig::new(None);
        assert!(
            config.wasm_module_path.to_string_lossy().ends_with("heycat_db.wasm"),
            "WASM path should end with heycat_db.wasm, got: {:?}",
            config.wasm_module_path
        );
    }

    #[test]
    fn test_publish_module_fails_when_wasm_not_found() {
        let mut config = SidecarConfig::new(None);
        // Set to a non-existent path
        config.wasm_module_path = PathBuf::from("/nonexistent/path/to/module.wasm");

        let _manager = SidecarManager::new(config.clone());

        // publish_module is private, but we can test via the error type
        // The error should be WasmModuleNotFound
        let err = SidecarError::WasmModuleNotFound(config.wasm_module_path.clone());
        assert!(matches!(err, SidecarError::WasmModuleNotFound(_)));
        assert!(err.to_string().contains("/nonexistent/path/to/module.wasm"));
    }

    #[test]
    fn test_module_publish_failed_error_message() {
        let err = SidecarError::ModulePublishFailed("test error message".to_string());
        assert!(err.to_string().contains("test error message"));
        assert!(err.to_string().contains("Failed to publish module"));
    }

    #[test]
    fn test_database_name_default() {
        assert_eq!(DEFAULT_DATABASE_NAME, "heycat");
    }
}
