use but_api_macros::but_api;
use but_core::sync::RepoExclusive;
use but_oplog::legacy::{OperationKind, SnapshotDetails, Trailer};
use but_rebase::graph_rebase::Editor;
use tracing::instrument;

use crate::commit::types::CommitDiscardResult;

/// Discard `subject_commit_id` using the behavior described by
/// [`commit_discard_only_with_perm()`].
#[but_api(crate::commit::json::CommitDiscardResult)]
pub fn commit_discard_only(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitDiscardResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_discard_only_with_perm(ctx, subject_commit_id, dry_run, guard.write_permission())
}

/// Discard `subject_commit_id` under caller-held exclusive repository access.
///
/// This materializes the discard rebase and returns the commit-ID mapping for
/// rewritten descendants. This variant does not create an oplog entry. For
/// lower-level implementation details, see
/// [`but_workspace::commit::discard_commit()`].
pub fn commit_discard_only_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitDiscardResult> {
    let mut meta = ctx.meta()?;
    let (repo, mut ws, _) = ctx.workspace_mut_and_db_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let rebase = but_workspace::commit::discard_commit(editor, subject_commit_id)?;

    let workspace =
        crate::workspace_state::from_successful_rebase(rebase, &repo, &mut cache, dry_run)?;

    Ok(CommitDiscardResult {
        discarded_commit: subject_commit_id,
        workspace,
    })
}

/// Discard `subject_commit_id` using the behavior described by
/// [`commit_discard_with_perm()`].
#[but_api(napi, crate::commit::json::CommitDiscardResult)]
#[instrument(err(Debug))]
pub fn commit_discard(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitDiscardResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_discard_with_perm(ctx, subject_commit_id, dry_run, guard.write_permission())
}

/// Discard `subject_commit_id` under caller-held exclusive repository access
/// and record an oplog snapshot on success.
///
/// This prepares a best-effort `DiscardCommit` oplog snapshot annotated with
/// `subject_commit_id`, discards the commit, and commits the snapshot only if
/// the operation succeeds. For lower-level implementation details, see
/// [`but_workspace::commit::discard_commit()`].
pub fn commit_discard_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitDiscardResult> {
    let details = SnapshotDetails::new(OperationKind::DiscardCommit).with_trailers(vec![Trailer {
        key: "sha".to_string(),
        value: subject_commit_id.to_string(),
    }]);
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        details,
        perm.read_permission(),
    )
    .ok();

    let res = commit_discard_only_with_perm(ctx, subject_commit_id, dry_run, perm);
    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);
    res
}
