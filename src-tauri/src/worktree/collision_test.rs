// Tests for collision detection module
//
// Testing philosophy: Focus on user-visible behaviors, not implementation details.
// These tests verify the collision detection workflow from a user's perspective.

use super::collision::{
    check_collision_at, cleanup_stale_lock, create_lock_at, format_collision_error, remove_lock_at,
    CollisionResult, LockInfo,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory for testing
fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a lock file with specified content
fn write_lock_file(path: &PathBuf, content: &str) {
    fs::write(path, content).expect("Failed to write lock file");
}

// =============================================================================
// Core Workflow Tests - Testing user-visible behaviors
// =============================================================================

#[test]
fn test_no_collision_when_lock_file_absent() {
    // User scenario: First time starting heycat in this directory
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");
    let data_dir = temp_dir.path().to_path_buf();

    let result = check_collision_at(&lock_file, &data_dir);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollisionResult::NoCollision);
}

#[test]
fn test_detects_running_instance_collision() {
    // User scenario: Another heycat instance is running
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");
    let data_dir = temp_dir.path().to_path_buf();

    // Use current PID - this process IS running
    let current_pid = std::process::id();
    write_lock_file(&lock_file, &format!("pid: {}\ntimestamp: 1234567890\n", current_pid));

    let result = check_collision_at(&lock_file, &data_dir);

    assert!(result.is_ok());
    match result.unwrap() {
        CollisionResult::InstanceRunning { pid, .. } => {
            assert_eq!(pid, current_pid);
        }
        other => panic!("Expected InstanceRunning, got {:?}", other),
    }
}

#[test]
fn test_detects_stale_lock_from_dead_process() {
    // User scenario: Previous heycat instance crashed, left lock file behind
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");
    let data_dir = temp_dir.path().to_path_buf();

    // Use a PID that's very unlikely to exist (max value)
    let dead_pid = 999999999u32;
    write_lock_file(&lock_file, &format!("pid: {}\ntimestamp: 1234567890\n", dead_pid));

    let result = check_collision_at(&lock_file, &data_dir);

    assert!(result.is_ok());
    match result.unwrap() {
        CollisionResult::StaleLock { lock_file: path } => {
            assert_eq!(path, lock_file);
        }
        other => panic!("Expected StaleLock, got {:?}", other),
    }
}

#[test]
fn test_malformed_lock_file_treated_as_stale() {
    // User scenario: Lock file corrupted somehow
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");
    let data_dir = temp_dir.path().to_path_buf();

    write_lock_file(&lock_file, "garbage content");

    let result = check_collision_at(&lock_file, &data_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), CollisionResult::StaleLock { .. }));
}

// =============================================================================
// Lock File Lifecycle Tests
// =============================================================================

#[test]
fn test_create_and_remove_lock_lifecycle() {
    // User scenario: Normal app startup and shutdown
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");
    let data_dir = temp_dir.path().to_path_buf();

    // Initially no lock
    assert!(!lock_file.exists());

    // Create lock on startup
    let created_path = create_lock_at(&lock_file);
    assert!(created_path.is_ok());
    assert!(lock_file.exists());

    // Lock should block other instances (current PID)
    let result = check_collision_at(&lock_file, &data_dir);
    assert!(matches!(result.unwrap(), CollisionResult::InstanceRunning { .. }));

    // Remove lock on shutdown
    let remove_result = remove_lock_at(&lock_file);
    assert!(remove_result.is_ok());
    assert!(!lock_file.exists());

    // No collision after removal
    let result = check_collision_at(&lock_file, &data_dir);
    assert_eq!(result.unwrap(), CollisionResult::NoCollision);
}

#[test]
fn test_cleanup_stale_lock_removes_file() {
    // User scenario: Auto-cleanup of stale lock from crashed instance
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");

    // Create a stale lock (dead PID)
    write_lock_file(&lock_file, "pid: 999999999\ntimestamp: 1234567890\n");
    assert!(lock_file.exists());

    // Cleanup stale lock
    let result = cleanup_stale_lock(&lock_file);
    assert!(result.is_ok());
    assert!(!lock_file.exists());
}

// =============================================================================
// LockInfo Parsing Tests
// =============================================================================

#[test]
fn test_lock_info_parse_valid_content() {
    let content = "pid: 12345\ntimestamp: 1703347200\n";
    let info = LockInfo::parse(content);

    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.pid, 12345);
    assert_eq!(info.timestamp, 1703347200);
}

#[test]
fn test_lock_info_parse_with_extra_whitespace() {
    let content = "pid:  12345  \ntimestamp:  1703347200  \n";
    let info = LockInfo::parse(content);

    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.pid, 12345);
    assert_eq!(info.timestamp, 1703347200);
}

#[test]
fn test_lock_info_parse_missing_fields() {
    // Missing timestamp
    assert!(LockInfo::parse("pid: 12345\n").is_none());

    // Missing pid
    assert!(LockInfo::parse("timestamp: 1703347200\n").is_none());

    // Empty content
    assert!(LockInfo::parse("").is_none());
}

#[test]
fn test_lock_info_serialize_roundtrip() {
    let original = LockInfo {
        pid: 42,
        timestamp: 1703347200,
        sidecar_pid: None,
    };

    let serialized = original.serialize();
    let parsed = LockInfo::parse(&serialized);

    assert!(parsed.is_some());
    assert_eq!(parsed.unwrap(), original);
}

#[test]
fn test_lock_info_serialize_roundtrip_with_sidecar() {
    let original = LockInfo {
        pid: 42,
        timestamp: 1703347200,
        sidecar_pid: Some(12345),
    };

    let serialized = original.serialize();
    assert!(serialized.contains("sidecar_pid: 12345"));

    let parsed = LockInfo::parse(&serialized);
    assert!(parsed.is_some());
    assert_eq!(parsed.unwrap(), original);
}

// =============================================================================
// Error Message Formatting Tests
// =============================================================================

#[test]
fn test_format_collision_error_for_running_instance() {
    let collision = CollisionResult::InstanceRunning {
        pid: 12345,
        data_dir: PathBuf::from("/home/user/.local/share/heycat"),
        lock_file: PathBuf::from("/home/user/.local/share/heycat/heycat.lock"),
    };

    let result = format_collision_error(&collision);
    assert!(result.is_some());

    let (title, message, steps) = result.unwrap();
    assert!(title.contains("Another instance"));
    assert!(message.contains("12345")); // PID in message
    assert!(!steps.is_empty()); // Resolution steps provided
}

#[test]
fn test_format_collision_error_for_stale_lock() {
    let collision = CollisionResult::StaleLock {
        lock_file: PathBuf::from("/home/user/.local/share/heycat/heycat.lock"),
    };

    let result = format_collision_error(&collision);
    assert!(result.is_some());

    let (title, _, steps) = result.unwrap();
    assert!(title.contains("Stale"));
    assert!(!steps.is_empty());
}

#[test]
fn test_format_collision_error_returns_none_for_no_collision() {
    let collision = CollisionResult::NoCollision;
    assert!(format_collision_error(&collision).is_none());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_remove_nonexistent_lock_succeeds() {
    // Removing a lock that doesn't exist should succeed silently
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("nonexistent.lock");

    let result = remove_lock_at(&lock_file);
    assert!(result.is_ok());
}

#[test]
fn test_lock_file_content_includes_required_fields() {
    let temp_dir = setup_temp_dir();
    let lock_file = temp_dir.path().join("heycat.lock");

    create_lock_at(&lock_file).expect("Failed to create lock");

    let content = fs::read_to_string(&lock_file).expect("Failed to read lock file");

    // Verify required fields are present
    assert!(content.contains("pid:"));
    assert!(content.contains("timestamp:"));

    // Verify we can parse it back
    let info = LockInfo::parse(&content);
    assert!(info.is_some());
    assert_eq!(info.unwrap().pid, std::process::id());
}
