//! General types for the APIs.

use std::collections::BTreeMap;

use but_workspace::RefInfo;

/// Represents the workspace for the frontend
///
/// This describes the post-operation workspace view that mutations should
/// return regardless of whether they executed for real or as a dry-run.
#[derive(Debug, Clone)]
pub struct WorkspaceState {
    /// Commits that were replaced by the operation. Maps `old_id -> new_id`.
    pub replaced_commits: BTreeMap<gix::ObjectId, gix::ObjectId>,
    /// The workspace presented for the frontend. See [`RefInfo`] for more
    /// detail.
    pub head_info: RefInfo,
}

impl WorkspaceState {
    /// Create a new workspace state from operation outputs.
    pub fn new(
        replaced_commits: BTreeMap<gix::ObjectId, gix::ObjectId>,
        head_info: RefInfo,
    ) -> Self {
        Self {
            replaced_commits,
            head_info,
        }
    }
}
