use std::collections::HashSet;

use but_api_macros::but_api;
use but_hunk_assignment::HunkAssignmentRequest;
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use but_rebase::graph_rebase::Editor;
use tracing::instrument;

use super::types::MoveChangesResult;

/// Uncommits changes from a commit (removes them from the commit tree) without
/// performing a checkout.
///
/// This acquires exclusive worktree access from `ctx` before extracting the
/// changes.
///
/// See [`commit_uncommit_changes_only_with_perm()`] for details.
#[but_api(crate::commit::json::MoveChangesResult)]
#[instrument(err(Debug))]
pub fn commit_uncommit_changes_only(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<but_core::DiffSpec>,
    assign_to: Option<but_core::ref_metadata::StackId>,
    dry_run: bool,
) -> anyhow::Result<MoveChangesResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_uncommit_changes_only_with_perm(
        ctx,
        commit_id,
        changes,
        assign_to,
        dry_run,
        guard.write_permission(),
    )
}

/// Extract `changes` from `commit_id` without performing a checkout, under
/// caller-held exclusive repository access.
///
/// The removed diff stays in the workspace as uncommitted changes. When
/// `assign_to` is set, newly surfaced hunks are reassigned to that stack after
/// the rebase is materialized. For lower-level implementation details, see
/// [`but_workspace::commit::uncommit_changes()`].
pub fn commit_uncommit_changes_only_with_perm(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<but_core::DiffSpec>,
    assign_to: Option<but_core::ref_metadata::StackId>,
    dry_run: bool,
    perm: &mut but_ctx::access::RepoExclusive,
) -> anyhow::Result<MoveChangesResult> {
    let context_lines = ctx.settings.context_lines;
    let (graph, preview_repo, replaced_commits) = {
        let mut meta = ctx.meta()?;
        let (repo, mut ws, mut db, _cache) =
            ctx.workspace_mut_and_db_mut_and_cache_with_perm(perm)?;

        let before_assignments = if !dry_run && assign_to.is_some() {
            let (assignments, _) = but_hunk_assignment::assignments_with_fallback(
                db.hunk_assignments_mut()?,
                &repo,
                &ws,
                None::<Vec<but_core::TreeChange>>,
                context_lines,
            )?;
            Some(assignments)
        } else {
            None
        };

        let editor = Editor::create(&mut ws, &mut meta, &repo)?;
        let outcome =
            but_workspace::commit::uncommit_changes(editor, commit_id, changes, context_lines)?;
        let graph = outcome.rebase.overlayed_graph()?;
        let replaced_commits = outcome.rebase.history.commit_mappings();
        let preview_repo = dry_run.then(|| outcome.rebase.repository().clone());

        if !dry_run {
            let materialized = outcome.rebase.materialize_without_checkout()?;

            if let (Some(before_assignments), Some(stack_id)) = (before_assignments, assign_to) {
                let (after_assignments, _) = but_hunk_assignment::assignments_with_fallback(
                    db.hunk_assignments_mut()?,
                    &repo,
                    materialized.workspace,
                    None::<Vec<but_core::TreeChange>>,
                    context_lines,
                )?;

                let to_assign = newly_surfaced_hunk_assignments(
                    before_assignments,
                    after_assignments,
                    stack_id,
                );

                but_hunk_assignment::assign(
                    db.hunk_assignments_mut()?,
                    &repo,
                    materialized.workspace,
                    to_assign,
                    context_lines,
                )?;
            }
        }

        (graph, preview_repo, replaced_commits)
    };
    let repo = preview_repo.unwrap_or_else(|| {
        ctx.repo
            .get()
            .expect("repo should remain available")
            .clone()
    });
    let mut cache = ctx.cache.get_cache_mut()?;

    Ok(MoveChangesResult {
        workspace: crate::workspace_state::from_overlayed_graph(
            graph,
            &repo,
            &mut cache,
            replaced_commits,
        )?,
    })
}

fn newly_surfaced_hunk_assignments(
    before_assignments: Vec<but_hunk_assignment::HunkAssignment>,
    after_assignments: Vec<but_hunk_assignment::HunkAssignment>,
    stack_id: but_core::ref_metadata::StackId,
) -> Vec<HunkAssignmentRequest> {
    let before_ids: HashSet<_> = before_assignments
        .into_iter()
        .filter_map(|assignment| assignment.id)
        .collect();

    after_assignments
        .into_iter()
        .filter(|assignment| assignment.id.is_some_and(|id| !before_ids.contains(&id)))
        .map(|assignment| HunkAssignmentRequest {
            hunk_header: assignment.hunk_header,
            path_bytes: assignment.path_bytes,
            stack_id: Some(stack_id),
            branch_ref_bytes: None,
        })
        .collect()
}

/// Extract `changes` from `commit_id` and record the rewrite in the oplog.
///
/// This acquires exclusive worktree access from `ctx` before extracting the
/// changes.
///
/// See [`commit_uncommit_changes_with_perm()`] for details.
#[but_api(napi, crate::commit::json::MoveChangesResult)]
#[instrument(err(Debug))]
pub fn commit_uncommit_changes(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<but_core::DiffSpec>,
    assign_to: Option<but_core::ref_metadata::StackId>,
    dry_run: bool,
) -> anyhow::Result<MoveChangesResult> {
    let mut guard = ctx.exclusive_worktree_access();
    commit_uncommit_changes_with_perm(
        ctx,
        commit_id,
        changes,
        assign_to,
        dry_run,
        guard.write_permission(),
    )
}

/// Extract `changes` from `commit_id` under caller-held exclusive repository
/// access and record an oplog snapshot on success.
///
/// When `assign_to` is set, newly surfaced hunks are assigned to that stack
/// after the rebase is materialized. This prepares a best-effort
/// `DiscardChanges` oplog snapshot and commits it only if the operation
/// succeeds. For lower-level implementation details, see
/// [`but_workspace::commit::uncommit_changes()`].
pub fn commit_uncommit_changes_with_perm(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    changes: Vec<but_core::DiffSpec>,
    assign_to: Option<but_core::ref_metadata::StackId>,
    dry_run: bool,
    perm: &mut but_ctx::access::RepoExclusive,
) -> anyhow::Result<MoveChangesResult> {
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        SnapshotDetails::new(OperationKind::DiscardChanges),
        perm.read_permission(),
    )
    .ok();

    let res =
        commit_uncommit_changes_only_with_perm(ctx, commit_id, changes, assign_to, dry_run, perm);

    crate::commit_oplog_snapshot_if_success(dry_run, maybe_oplog_entry, ctx, perm, &res);

    res
}
