use std::fs;
use std::path::{Path, PathBuf};

/// Default settings file name used in main repository
pub const DEFAULT_SETTINGS_FILE: &str = "settings.json";

/// Context for a git worktree, providing isolation identifier
#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeContext {
    /// Human-readable identifier derived from worktree directory name
    pub identifier: String,
    /// Full path to the worktree's gitdir (inside main repo's .git/worktrees/)
    pub gitdir_path: PathBuf,
}

impl WorktreeContext {
    /// Returns the worktree-specific settings file name.
    ///
    /// Format: `settings-{identifier}.json`
    pub fn settings_file_name(&self) -> String {
        format!("settings-{}.json", self.identifier)
    }
}

/// State wrapper for worktree context, managed by Tauri.
/// Fields are consumed by dependent specs (worktree-paths, worktree-config).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorktreeState {
    pub context: Option<WorktreeContext>,
}

impl WorktreeState {
    /// Returns the appropriate settings file name based on worktree context.
    ///
    /// - Returns `settings-{identifier}.json` when running in a worktree
    /// - Returns `settings.json` when running in main repository
    pub fn settings_file_name(&self) -> String {
        match &self.context {
            Some(ctx) => ctx.settings_file_name(),
            None => DEFAULT_SETTINGS_FILE.to_string(),
        }
    }
}

/// Detects if the application is running from a git worktree directory.
///
/// Git worktrees have a `.git` file (not directory) containing a `gitdir:` reference
/// to the actual git directory inside the main repository's `.git/worktrees/` folder.
///
/// # Returns
/// - `Some(WorktreeContext)` if running in a worktree, with identifier and gitdir path
/// - `None` if running in main repository or detection fails
pub fn detect_worktree() -> Option<WorktreeContext> {
    detect_worktree_at(Path::new(".git"))
}

/// Detects worktree context at a specific .git path (for testing)
pub fn detect_worktree_at(git_path: &Path) -> Option<WorktreeContext> {
    // Check if .git exists and is a file (worktrees have .git as a file, not directory)
    let metadata = fs::metadata(git_path).ok()?;
    if metadata.is_dir() {
        return None; // Main repository - .git is a directory
    }

    // Read and parse the .git file
    let content = fs::read_to_string(git_path).ok()?;
    let gitdir = content
        .lines()
        .find(|line| line.starts_with("gitdir: "))?
        .strip_prefix("gitdir: ")?
        .trim();

    if gitdir.is_empty() {
        return None;
    }

    let gitdir_path = PathBuf::from(gitdir);

    // Extract worktree name from gitdir path
    // gitdir typically looks like: /path/to/repo/.git/worktrees/feature-name
    let identifier = extract_worktree_name(&gitdir_path)?;

    Some(WorktreeContext {
        identifier,
        gitdir_path,
    })
}

/// Extracts the worktree name from a gitdir path.
///
/// The gitdir path format is: `/path/to/repo/.git/worktrees/<worktree-name>`
/// We extract the last component as the identifier.
fn extract_worktree_name(gitdir_path: &Path) -> Option<String> {
    // The worktree name is the last component of the gitdir path
    let name = gitdir_path.file_name()?.to_str()?;

    // Sanity check: worktree names shouldn't be empty or just dots
    if name.is_empty() || name == "." || name == ".." {
        return None;
    }

    Some(name.to_string())
}
