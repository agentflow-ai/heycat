// Tests for worktree detection
// Test cases:
// - Returns None when .git is a directory (main repo)
// - Returns worktree identifier when .git is a file with valid gitdir content
// - Returns None when .git file is malformed
// - Same worktree path always generates same identifier (deterministic)
// - Different worktree paths generate different identifiers

use super::{detect_worktree_at, WorktreeContext};
use std::fs;
use tempfile::TempDir;

/// Helper to create a temp directory simulating a main repo (.git as directory)
fn create_main_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    temp_dir
}

/// Helper to create a temp directory simulating a worktree (.git as file)
fn create_worktree(worktree_name: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let git_file = temp_dir.path().join(".git");
    let gitdir_content = format!(
        "gitdir: /path/to/repo/.git/worktrees/{}\n",
        worktree_name
    );
    fs::write(&git_file, gitdir_content).unwrap();
    temp_dir
}

#[test]
fn test_returns_none_for_main_repo() {
    let temp_dir = create_main_repo();
    let git_path = temp_dir.path().join(".git");

    let result = detect_worktree_at(&git_path);

    assert!(result.is_none(), "Should return None for main repo where .git is a directory");
}

#[test]
fn test_returns_context_for_valid_worktree() {
    let temp_dir = create_worktree("feature-branch");
    let git_path = temp_dir.path().join(".git");

    let result = detect_worktree_at(&git_path);

    assert!(result.is_some(), "Should return Some for valid worktree");
    let context = result.unwrap();
    assert_eq!(context.identifier, "feature-branch");
    assert_eq!(
        context.gitdir_path.to_str().unwrap(),
        "/path/to/repo/.git/worktrees/feature-branch"
    );
}

#[test]
fn test_returns_none_for_malformed_git_file() {
    let temp_dir = TempDir::new().unwrap();
    let git_file = temp_dir.path().join(".git");

    // Write malformed content (no gitdir: prefix)
    fs::write(&git_file, "invalid content\n").unwrap();

    let result = detect_worktree_at(&git_file);

    assert!(result.is_none(), "Should return None for malformed .git file");
}

#[test]
fn test_returns_none_for_empty_gitdir() {
    let temp_dir = TempDir::new().unwrap();
    let git_file = temp_dir.path().join(".git");

    // Write gitdir with empty path
    fs::write(&git_file, "gitdir: \n").unwrap();

    let result = detect_worktree_at(&git_file);

    assert!(result.is_none(), "Should return None for empty gitdir path");
}

#[test]
fn test_returns_none_for_missing_git() {
    let temp_dir = TempDir::new().unwrap();
    let git_path = temp_dir.path().join(".git");
    // Don't create .git - it doesn't exist

    let result = detect_worktree_at(&git_path);

    assert!(result.is_none(), "Should return None when .git doesn't exist");
}

#[test]
fn test_same_path_generates_same_identifier() {
    let temp_dir = create_worktree("my-feature");
    let git_path = temp_dir.path().join(".git");

    let result1 = detect_worktree_at(&git_path);
    let result2 = detect_worktree_at(&git_path);

    assert_eq!(result1, result2, "Same path should generate same identifier");
}

#[test]
fn test_different_paths_generate_different_identifiers() {
    let temp_dir1 = create_worktree("feature-a");
    let temp_dir2 = create_worktree("feature-b");
    let git_path1 = temp_dir1.path().join(".git");
    let git_path2 = temp_dir2.path().join(".git");

    let result1 = detect_worktree_at(&git_path1).unwrap();
    let result2 = detect_worktree_at(&git_path2).unwrap();

    assert_ne!(
        result1.identifier, result2.identifier,
        "Different worktrees should have different identifiers"
    );
}

#[test]
fn test_handles_worktree_with_special_characters_in_name() {
    let temp_dir = create_worktree("feature_with-special.chars");
    let git_path = temp_dir.path().join(".git");

    let result = detect_worktree_at(&git_path);

    assert!(result.is_some());
    assert_eq!(result.unwrap().identifier, "feature_with-special.chars");
}

#[test]
fn test_trims_whitespace_from_gitdir() {
    let temp_dir = TempDir::new().unwrap();
    let git_file = temp_dir.path().join(".git");

    // Write gitdir with extra whitespace
    fs::write(&git_file, "gitdir:   /path/to/worktrees/my-branch   \n").unwrap();

    let result = detect_worktree_at(&git_file);

    assert!(result.is_some());
    assert_eq!(result.unwrap().identifier, "my-branch");
}
