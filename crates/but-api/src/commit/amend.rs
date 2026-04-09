use but_api_macros::but_api;
use but_core::{DiffSpec, sync::RepoExclusive};
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use but_rebase::graph_rebase::{Editor, LookupStep as _};
use tracing::instrument;

use super::types::CommitCreateResult;

/// Amends the commit at `commit_id` with `changes`.
///
/// See [`but_workspace::commit::commit_amend()`] for lower-level implementation
/// details.
#[but_api(crate::commit::json::CommitCreateResult)]
#[instrument(err(Debug))]
pub fn commit_amend_only(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<DiffSpec>,
    dry_run: bool,
) -> anyhow::Result<CommitCreateResult> {
    let context_lines = ctx.settings.context_lines;
    let mut guard = ctx.exclusive_worktree_access();
    commit_amend_only_impl(
        ctx,
        commit_id,
        changes,
        dry_run,
        context_lines,
        guard.write_permission(),
    )
}

pub(crate) fn commit_amend_only_impl(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<DiffSpec>,
    dry_run: bool,
    context_lines: u32,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitCreateResult> {
    let mut meta = ctx.meta()?;
    let (repo, mut ws, _, _cache) = ctx.workspace_mut_and_db_and_cache_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let but_workspace::commit::CommitAmendOutcome {
        rebase,
        commit_selector,
        rejected_specs,
    } = but_workspace::commit::commit_amend(editor, commit_id, changes, context_lines)?;

    let new_commit = commit_selector
        .map(|commit_selector| rebase.lookup_pick(commit_selector))
        .transpose()?;
    let workspace =
        crate::workspace_state::from_successful_rebase(rebase, &repo, &mut cache, dry_run)?;

    Ok(CommitCreateResult {
        new_commit,
        rejected_specs,
        workspace,
    })
}

/// Amend the commit at `commit_id` with `changes` and record an oplog snapshot on success.
///
/// This performs the rewrite under exclusive worktree access and creates a
/// best-effort `AmendCommit` oplog entry if the operation succeeds. For
/// lower-level implementation details, see
/// [`but_workspace::commit::commit_amend()`].
#[but_api(napi, crate::commit::json::CommitCreateResult)]
#[instrument(err(Debug))]
pub fn commit_amend(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<DiffSpec>,
    dry_run: bool,
) -> anyhow::Result<CommitCreateResult> {
    let context_lines = ctx.settings.context_lines;
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details(
        ctx,
        SnapshotDetails::new(OperationKind::AmendCommit),
    )
    .ok();

    let mut guard = ctx.exclusive_worktree_access();
    let res = commit_amend_only_impl(
        ctx,
        commit_id,
        changes,
        dry_run,
        context_lines,
        guard.write_permission(),
    );
    crate::commit_oplog_snapshot_if_success(
        dry_run,
        maybe_oplog_entry,
        ctx,
        guard.write_permission(),
        &res,
    );
    res
}
