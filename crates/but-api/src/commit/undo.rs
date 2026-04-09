use anyhow::Context as _;
use but_api_macros::but_api;
use but_core::{DiffSpec, sync::RepoExclusive};
use but_oplog::legacy::{OperationKind, SnapshotDetails, Trailer};
use but_rebase::graph_rebase::{Editor, LookupStep as _};
use tracing::instrument;

use crate::commit::types::CommitUndoResult;

/// Undo `subject_commit_id` using the behavior described by
/// [`commit_undo_only_with_perm()`].
#[but_api(napi, crate::commit::json::CommitUndoResult)]
#[instrument(err(Debug))]
pub fn commit_undo(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitUndoResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_undo_with_perm(ctx, subject_commit_id, dry_run, guard.write_permission())
}

/// Undo `subject_commit_id` using the behavior described by
/// [`commit_undo_only_with_perm()`].
pub fn commit_undo_only(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
) -> anyhow::Result<CommitUndoResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_undo_only_with_perm(ctx, subject_commit_id, dry_run, guard.write_permission())
}

/// Undo `subject_commit_id` using the behavior described by
/// [`commit_undo_only_with_perm()`].
pub fn commit_undo_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitUndoResult> {
    let details = SnapshotDetails::new(OperationKind::UndoCommit).with_trailers(vec![Trailer {
        key: "sha".to_string(),
        value: subject_commit_id.to_string(),
    }]);
    let maybe_oplog_entry = if dry_run {
        None
    } else {
        but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
            ctx,
            details,
            perm.read_permission(),
        )
        .ok()
    };

    let res = commit_undo_only_with_perm(ctx, subject_commit_id, dry_run, perm);
    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);
    res
}

/// Undo `subject_commit_id`, under caller-held exclusive repository access.
///
/// This will move the changes in the commit to be unassigned and discard the commit.
pub fn commit_undo_only_with_perm(
    ctx: &mut but_ctx::Context,
    subject_commit_id: gix::ObjectId,
    dry_run: bool,
    perm: &mut RepoExclusive,
) -> anyhow::Result<CommitUndoResult> {
    let context_lines = ctx.settings.context_lines;

    let changes = {
        let repo = ctx.repo.get()?;
        let commit = repo.find_commit(subject_commit_id)?;

        let mut parent_ids = commit.parent_ids();
        let first_parent = parent_ids.next().map(|id| id.detach());

        // TODO: do we want to handle this?
        anyhow::ensure!(
            parent_ids.next().is_none(),
            "expected {} to have at most one parent",
            subject_commit_id.to_hex()
        );

        let changes = but_core::diff::tree_changes(&repo, first_parent, subject_commit_id)?;
        changes.into_iter().map(DiffSpec::from).collect::<Vec<_>>()
    };

    let mut meta = ctx.meta()?;
    let (repo, mut ws, _) = ctx.workspace_mut_and_db_with_perm(perm)?;
    let mut cache = ctx.cache.get_cache_mut()?;
    let editor = Editor::create(&mut ws, &mut meta, &repo)?;

    let final_rebase = if changes.is_empty() {
        but_workspace::commit::discard_commit(editor, subject_commit_id)
            .with_context(|| format!("failed to discard {}", subject_commit_id.to_hex()))?
    } else {
        let but_workspace::commit::UncommitChangesOutcome {
            rebase,
            commit_selector,
        } = but_workspace::commit::uncommit_changes(
            editor,
            subject_commit_id,
            changes,
            context_lines,
        )?;
        let new_commit = rebase.lookup_pick(commit_selector)?;
        let editor = rebase.into_editor();
        but_workspace::commit::discard_commit(editor, new_commit)
            .with_context(|| format!("failed to discard {}", subject_commit_id.to_hex()))?
    };

    let workspace = crate::workspace_state::from_rebase_preview(
        &final_rebase,
        &mut cache,
        final_rebase.history.commit_mappings(),
    )?;

    if !dry_run {
        let _ = final_rebase.materialize()?;
    }

    Ok(CommitUndoResult {
        undone_commit: subject_commit_id,
        workspace,
    })
}
