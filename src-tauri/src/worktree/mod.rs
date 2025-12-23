mod detector;

pub use detector::{detect_worktree, WorktreeContext, WorktreeState};

#[cfg(test)]
pub use detector::detect_worktree_at;

#[cfg(test)]
mod detector_test;
