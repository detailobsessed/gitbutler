use but_api_macros::but_api;
use but_core::sync::RepoExclusive;
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use but_rebase::graph_rebase::{
    Editor,
    mutate::{InsertSide, RelativeTo},
};
use tracing::instrument;

use super::types::CommitMoveResult;

/// Moves `subject_commit_id` to `side` of `relative_to`.
///
/// This acquires exclusive worktree access from `ctx` before moving the
/// commit.
///
/// For details, see [`commit_move_only_with_perm()`].
#[but_api(crate::commit::json::CommitMoveResult)]
pub fn commit_move_only(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    #[but_api(crate::commit::json::RelativeTo)] relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
) -> anyhow::Result<CommitMoveResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_move_only_with_perm(
        ctx,
        subject_commit_id,
        relative_to,
        side,
        dry_run,
        guard.write_permission(),
    )
}

/// Move `subject_commit_id` to the `side` of `relative_to` under
/// caller-held exclusive repository access.
///
/// This returns the post-operation workspace view without creating an oplog
/// entry. For lower-level implementation details, see
/// [`but_workspace::commit::move_commit()`].
pub fn commit_move_only_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitMoveResult> {
    let mut meta = ctx.meta()?;
    let (repo, mut ws, _) = ctx.workspace_mut_and_db_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let rebase = but_workspace::commit::move_commit(editor, subject_commit_id, relative_to, side)?;

    Ok(CommitMoveResult {
        workspace: crate::workspace_state::from_successful_rebase(
            rebase, &repo, &mut cache, dry_run,
        )?,
    })
}

/// Moves `subject_commit_id` to `side` of `relative_to` and records an oplog
/// snapshot on success.
///
/// This acquires exclusive worktree access from `ctx` before moving the
/// commit.
///
/// For details, see [`commit_move_with_perm()`].
#[but_api(napi, crate::commit::json::CommitMoveResult)]
#[instrument(err(Debug))]
pub fn commit_move(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    #[but_api(crate::commit::json::RelativeTo)] relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
) -> anyhow::Result<CommitMoveResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_move_with_perm(
        ctx,
        subject_commit_id,
        relative_to,
        side,
        dry_run,
        guard.write_permission(),
    )
}

/// Moves `subject_commit_id` to `side` of `relative_to` under caller-held
/// exclusive repository access and records an oplog snapshot on success.
///
/// It prepares a best-effort `MoveCommit` oplog snapshot, performs the move,
/// and commits the snapshot only if the operation succeeds. For lower-level
/// implementation details, see [`but_workspace::commit::move_commit()`].
pub fn commit_move_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitMoveResult> {
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        SnapshotDetails::new(OperationKind::MoveCommit),
        perm.read_permission(),
    )
    .ok();

    let res = commit_move_only_with_perm(ctx, subject_commit_id, relative_to, side, dry_run, perm);
    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);
    res
}
