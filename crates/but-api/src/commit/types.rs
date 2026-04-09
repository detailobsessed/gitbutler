use but_core::{DiffSpec, tree::create_tree::RejectionReason};

/// Outcome after creating a commit.
pub struct CommitCreateResult {
    /// If the commit was successfully created. This should only be none if all the DiffSpecs were rejected.
    pub new_commit: Option<gix::ObjectId>,
    /// Any specs that failed to be committed.
    pub rejected_specs: Vec<(RejectionReason, DiffSpec)>,
    /// Workspace state after the create or amend.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome after moving changes between commits.
pub struct MoveChangesResult {
    /// Workspace state after moving changes.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome after rewording a commit.
pub struct CommitRewordResult {
    /// The ID of the newly created commit with the updated message.
    pub new_commit: gix::ObjectId,
    /// Workspace state after the reword.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome of squashing one commit into another.
pub struct CommitSquashResult {
    /// The ID of the newly created squashed commit.
    pub new_commit: gix::ObjectId,
    /// Workspace state after the squash.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome of moving a commit.
pub struct CommitMoveResult {
    /// Workspace state after the move.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome after inserting a blank commit.
pub struct CommitInsertBlankResult {
    /// The ID of the newly inserted blank commit.
    pub new_commit: gix::ObjectId,
    /// Workspace state after inserting the blank commit.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome of discarding a commit.
pub struct CommitDiscardResult {
    /// The ID of the commit discarded.
    pub discarded_commit: gix::ObjectId,
    /// Workspace state after discarding the commit.
    pub workspace: crate::types::WorkspaceState,
}

/// Outcome of undoing a commit.
pub struct CommitUndoResult {
    /// The ID of the commit that was undone.
    pub undone_commit: gix::ObjectId,
    /// Workspace state after undoing the commit.
    pub workspace: crate::types::WorkspaceState,
}
