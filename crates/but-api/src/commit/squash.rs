use but_api_macros::but_api;
use but_core::sync::RepoExclusive;
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use but_rebase::graph_rebase::{Editor, LookupStep as _};
use tracing::instrument;

use super::types::CommitSquashResult;

/// Squash `subject_commit_id` into `target_commit_id`.
///
/// This acquires exclusive worktree access from `ctx` before rewriting the
/// commits.
///
/// For details, see [`commit_squash_only_with_perm()`].
#[but_api(crate::commit::json::CommitSquashResult)]
#[instrument(err(Debug))]
pub fn commit_squash_only(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    target_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitSquashResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_squash_only_with_perm(
        ctx,
        subject_commit_id,
        target_commit_id,
        dry_run,
        guard.write_permission(),
    )
}

/// Squash `subject_commit_id` into `target_commit_id` under caller-held
/// exclusive repository access.
///
/// This materializes the squash rebase and returns the resulting squashed
/// commit ID together with rewritten commit mappings. This variant does not
/// create an oplog entry. For lower-level implementation details, see
/// [`but_workspace::commit::squash_commits()`].
pub fn commit_squash_only_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    target_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitSquashResult> {
    let mut meta = ctx.meta()?;
    let (repo, mut ws, _) = ctx.workspace_mut_and_db_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let outcome =
        but_workspace::commit::squash_commits(editor, subject_commit_id, target_commit_id)?;

    let new_commit = outcome.rebase.lookup_pick(outcome.commit_selector)?;
    let workspace =
        crate::workspace_state::from_successful_rebase(outcome.rebase, &repo, &mut cache, dry_run)?;

    Ok(CommitSquashResult {
        new_commit,
        workspace,
    })
}

/// Squash `subject_commit_id` into `target_commit_id` and record an oplog
/// snapshot on success.
///
/// This acquires exclusive worktree access from `ctx` before rewriting the
/// commits.
///
/// For details, see [`commit_squash_with_perm()`].
#[but_api(napi, crate::commit::json::CommitSquashResult)]
#[instrument(err(Debug))]
pub fn commit_squash(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    target_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitSquashResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_squash_with_perm(
        ctx,
        subject_commit_id,
        target_commit_id,
        dry_run,
        guard.write_permission(),
    )
}

/// Squash `subject_commit_id` into `target_commit_id` under caller-held
/// exclusive repository access and record an oplog snapshot on success.
///
/// It prepares a best-effort `SquashCommit` oplog snapshot, performs the
/// squash, and commits the snapshot only if the operation succeeds. For
/// lower-level implementation details, see
/// [`but_workspace::commit::squash_commits()`].
pub fn commit_squash_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    target_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitSquashResult> {
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        SnapshotDetails::new(OperationKind::SquashCommit),
        perm.read_permission(),
    )
    .ok();

    let res = commit_squash_only_with_perm(ctx, subject_commit_id, target_commit_id, dry_run, perm);
    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);
    res
}
