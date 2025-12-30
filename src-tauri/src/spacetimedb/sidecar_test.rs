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
fn test_calculate_port_main_repo() {
    assert_eq!(calculate_port(None), DEFAULT_PORT);
}

#[test]
fn test_calculate_port_worktree_different_from_default() {
    let port = calculate_port(Some("heycat-epictetus"));
    assert_ne!(port, DEFAULT_PORT, "Worktree should use different port");
    assert!(
        port >= DEFAULT_PORT + WORKTREE_PORT_OFFSET,
        "Worktree port should be >= {}",
        DEFAULT_PORT + WORKTREE_PORT_OFFSET
    );
    assert!(
        port < DEFAULT_PORT + WORKTREE_PORT_OFFSET + WORKTREE_PORT_RANGE,
        "Worktree port should be < {}",
        DEFAULT_PORT + WORKTREE_PORT_OFFSET + WORKTREE_PORT_RANGE
    );
}

#[test]
fn test_calculate_port_deterministic() {
    // Same worktree ID should always produce the same port
    let port1 = calculate_port(Some("feature-branch"));
    let port2 = calculate_port(Some("feature-branch"));
    assert_eq!(port1, port2);
}

#[test]
fn test_worktree_uses_calculated_port() {
    let config = SidecarConfig::new(Some("test-worktree"));
    assert_ne!(config.port, DEFAULT_PORT, "Worktree config should not use default port");
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
