mod collision;
mod detector;

// WorktreeContext exported for dependent specs (worktree-paths, worktree-config)
#[allow(unused_imports)]
pub use detector::{detect_worktree, WorktreeContext, WorktreeState, DEFAULT_SETTINGS_FILE};

// Collision detection exports for worktree-collision-detection spec
// Used by lib.rs setup() for startup collision checks
#[allow(unused_imports)]
pub use collision::{
    check_collision, cleanup_stale_lock, create_lock, format_collision_error, remove_lock,
    update_lock_with_sidecar_pid, CollisionError, CollisionResult,
};

#[cfg(test)]
#[allow(unused_imports)]
pub use collision::{check_collision_at, create_lock_at, remove_lock_at, LockInfo};

#[cfg(test)]
pub use detector::detect_worktree_at;

#[cfg(test)]
mod collision_test;

#[cfg(test)]
mod detector_test;
