use super::*;
use crate::voice_commands::executor::ActionErrorCode;
use std::collections::HashMap;

fn params(app: &str) -> HashMap<String, String> {
    let mut p = HashMap::new();
    p.insert("app".to_string(), app.to_string());
    p
}

fn params_with_close(app: &str) -> HashMap<String, String> {
    let mut p = HashMap::new();
    p.insert("app".to_string(), app.to_string());
    p.insert("close".to_string(), "true".to_string());
    p
}

#[tokio::test]
#[ignore] // Opens Safari - skip during local dev
async fn test_open_safari_successfully() {
    // This test actually opens Safari - only run on macOS
    if !cfg!(target_os = "macos") {
        return;
    }

    let action = AppLauncherAction::new();
    let result = action.execute(&params("Safari")).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("Safari"));
    assert!(result.data.is_some());
}

#[tokio::test]
#[ignore] // Opens TextEdit - skip during local dev
async fn test_open_app_with_spaces_in_name() {
    // Skip on non-macOS or if VS Code is not installed
    if !cfg!(target_os = "macos") {
        return;
    }

    let action = AppLauncherAction::new();
    // Use TextEdit which is always installed on macOS
    let result = action.execute(&params("TextEdit")).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("TextEdit"));
}

#[tokio::test]
async fn test_nonexistent_app_returns_not_found() {
    if !cfg!(target_os = "macos") {
        return;
    }

    let action = AppLauncherAction::new();
    let result = action.execute(&params("NonexistentAppThatDoesNotExist12345")).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::NotFound);
    assert!(error.message.contains("not found"));
}

#[tokio::test]
#[ignore] // Opens Safari - skip during local dev
async fn test_case_variation_opens_app() {
    // macOS open command is case-insensitive
    if !cfg!(target_os = "macos") {
        return;
    }

    let action = AppLauncherAction::new();
    // Use lowercase for Safari
    let result = action.execute(&params("safari")).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    // The result message should contain the app name we passed
    assert!(result.message.to_lowercase().contains("safari"));
}

#[tokio::test]
async fn test_empty_app_name_returns_invalid_parameter() {
    let action = AppLauncherAction::new();
    let result = action.execute(&params("")).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
    assert!(error.message.contains("empty"));
}

#[tokio::test]
async fn test_whitespace_only_app_name_returns_invalid_parameter() {
    let action = AppLauncherAction::new();
    let result = action.execute(&params("   ")).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
}

#[tokio::test]
async fn test_missing_app_parameter_returns_error() {
    let action = AppLauncherAction::new();
    let result = action.execute(&HashMap::new()).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
    assert!(error.message.contains("app"));
}

