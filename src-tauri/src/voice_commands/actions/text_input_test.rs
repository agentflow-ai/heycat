use super::*;
use crate::voice_commands::executor::ActionErrorCode;
use std::collections::HashMap;

fn params(text: &str) -> HashMap<String, String> {
    let mut p = HashMap::new();
    p.insert("text".to_string(), text.to_string());
    p
}

fn params_with_delay(text: &str, delay_ms: u64) -> HashMap<String, String> {
    let mut p = HashMap::new();
    p.insert("text".to_string(), text.to_string());
    p.insert("delay_ms".to_string(), delay_ms.to_string());
    p
}

#[tokio::test]
async fn test_empty_text_returns_success() {
    let action = TextInputAction::new();
    let result = action.execute(&params("")).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("No text"));
}

#[tokio::test]
async fn test_missing_text_parameter_returns_error() {
    let action = TextInputAction::new();
    let result = action.execute(&HashMap::new()).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
    assert!(error.message.contains("text"));
}

#[tokio::test]
#[cfg(target_os = "macos")]
#[ignore] // Types into active window - skip during local dev
async fn test_type_hello_world() {
    // This test requires Accessibility permission to be granted
    // If permission is not granted, we expect PERMISSION_DENIED error
    let action = TextInputAction::new();
    let result = action.execute(&params("hello world")).await;

    // Either succeeds (permission granted) or returns permission error
    match result {
        Ok(r) => {
            assert!(r.message.contains("11")); // "hello world" has 11 characters
        }
        Err(e) => {
            assert_eq!(e.code, ActionErrorCode::PermissionDenied);
        }
    }
}

#[tokio::test]
#[cfg(target_os = "macos")]
#[ignore] // Types into active window - skip during local dev
async fn test_type_special_characters() {
    let action = TextInputAction::new();
    let result = action.execute(&params("!@#$%")).await;

    match result {
        Ok(r) => {
            assert!(r.message.contains("5")); // 5 special characters
        }
        Err(e) => {
            assert_eq!(e.code, ActionErrorCode::PermissionDenied);
        }
    }
}

#[tokio::test]
#[cfg(target_os = "macos")]
#[ignore] // Types into active window - skip during local dev
async fn test_type_unicode_characters() {
    let action = TextInputAction::new();
    let result = action.execute(&params("héllo 🎉")).await;

    match result {
        Ok(r) => {
            // "héllo 🎉" has 7 characters
            assert!(r.message.contains("7"));
        }
        Err(e) => {
            assert_eq!(e.code, ActionErrorCode::PermissionDenied);
        }
    }
}

#[tokio::test]
#[cfg(target_os = "macos")]
#[ignore] // Types into active window - skip during local dev
async fn test_configurable_delay() {
    let action = TextInputAction::new();
    // Test with custom delay
    let result = action.execute(&params_with_delay("ab", 50)).await;

    match result {
        Ok(r) => {
            assert!(r.message.contains("2"));
        }
        Err(e) => {
            assert_eq!(e.code, ActionErrorCode::PermissionDenied);
        }
    }
}

#[test]
fn test_check_accessibility_permission_callable() {
    // Just verify the function is callable (doesn't panic)
    let _result = check_accessibility_permission();
}

#[tokio::test]
#[cfg(not(target_os = "macos"))]
async fn test_non_macos_returns_unsupported() {
    let action = TextInputAction::new();
    let result = action.execute(&params("hello")).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::UnsupportedPlatform);
}
