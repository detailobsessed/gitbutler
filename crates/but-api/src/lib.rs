//! The API layer is what can be used to create GitButler applications.
//!
//! ### Coordinating Filesystem Access
//!
//! For them to behave correctly in multi-threaded scenarios, be sure to use an *exclusive or shared* lock
//! on this level.
//! Lower-level crates like `but-workspace` won't use filesystem-based locking beyond what Git offers natively.
#![cfg_attr(not(feature = "napi"), forbid(unsafe_code))]
#![cfg_attr(feature = "napi", deny(unsafe_code))]
#![deny(missing_docs)]

#[cfg(feature = "legacy")]
pub mod legacy;

/// Functions for GitHub authentication.
pub mod github;

/// Functions for GitLab authentication.
pub mod gitlab;

/// Functions that take a branch as input.
pub mod branch;

/// Functions that operate commits
pub mod commit;

/// Functions that show what changed in various Git entities, like trees, commits and the worktree.
pub mod diff;

/// Types meant to be serialised to JSON, without degenerating information despite the need to be UTF-8 encodable.
/// EXPERIMENTAL
pub mod json;

/// Functions releated to platform detection and information.
pub mod platform;

pub mod panic_capture;

/// The types for watcher events
#[cfg(feature = "export-schema")]
pub mod watcher;

pub mod types;

mod workspace_state;

pub(crate) fn should_commit_oplog_snapshot(dry_run: bool) -> bool {
    !dry_run
}

pub(crate) fn commit_oplog_snapshot_if_success<T, E>(
    dry_run: bool,
    maybe_snapshot: Option<but_oplog::UnmaterializedOplogSnapshot>,
    ctx: &mut but_ctx::Context,
    perm: &mut but_ctx::access::RepoExclusive,
    result: &Result<T, E>,
) {
    if should_commit_oplog_snapshot(dry_run)
        && let Some(snapshot) = maybe_snapshot.filter(|_| result.is_ok())
    {
        snapshot.commit(ctx, perm).ok();
    }
}
