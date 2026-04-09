use but_api_macros::but_api;
use but_core::sync::RepoExclusive;
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use but_rebase::graph_rebase::{
    Editor, LookupStep as _,
    mutate::{InsertSide, RelativeTo},
};
use tracing::instrument;

use super::types::CommitInsertBlankResult;

/// Inserts a blank commit on `side` of `relative_to`.
///
/// `side` chooses whether the blank commit lands before or after `relative_to`.
#[but_api(crate::commit::json::CommitInsertBlankResult)]
#[instrument(err(Debug))]
pub fn commit_insert_blank_only(
    ctx: &mut but_ctx::Context,
    #[but_api(crate::commit::json::RelativeTo)] relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
) -> anyhow::Result<CommitInsertBlankResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_insert_blank_only_impl(ctx, relative_to, side, dry_run, guard.write_permission())
}

pub(crate) fn commit_insert_blank_only_impl(
    ctx: &mut but_ctx::Context,
    relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitInsertBlankResult> {
    let mut meta = ctx.meta()?;
    let (repo, mut ws, _) = ctx.workspace_mut_and_db_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let (rebase, blank_commit_selector) =
        but_workspace::commit::insert_blank_commit(editor, side, relative_to)?;
    let new_commit = rebase.lookup_pick(blank_commit_selector)?;
    let workspace =
        crate::workspace_state::from_successful_rebase(rebase, &repo, &mut cache, dry_run)?;

    Ok(CommitInsertBlankResult {
        new_commit,
        workspace,
    })
}

/// Inserts a blank commit on `side` of `relative_to` and records an oplog
/// snapshot on success.
///
/// For details, see [`commit_insert_blank_with_perm()`].
#[but_api(napi, crate::commit::json::CommitInsertBlankResult)]
#[instrument(err(Debug))]
pub fn commit_insert_blank(
    ctx: &mut but_ctx::Context,
    #[but_api(crate::commit::json::RelativeTo)] relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
) -> anyhow::Result<CommitInsertBlankResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_insert_blank_with_perm(ctx, relative_to, side, dry_run, guard.write_permission())
}

/// Create an empty commit next to `relative_to` under caller-held exclusive
/// repository access and record an oplog snapshot on success.
///
/// `side` chooses whether the blank commit lands before or after `relative_to`.
/// This prepares a best-effort `InsertBlankCommit` oplog snapshot, creates the
/// commit, and commits the snapshot only if the operation succeeds. For
/// lower-level implementation details, see
/// [`but_workspace::commit::insert_blank_commit()`].
pub fn commit_insert_blank_with_perm(
    ctx: &mut but_ctx::Context,
    relative_to: RelativeTo,
    side: InsertSide,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitInsertBlankResult> {
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        SnapshotDetails::new(OperationKind::InsertBlankCommit),
        perm.read_permission(),
    )
    .ok();

    let res = commit_insert_blank_only_impl(ctx, relative_to, side, dry_run, perm);
    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);
    res
}
