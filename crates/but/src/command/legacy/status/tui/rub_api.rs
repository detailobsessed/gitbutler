//! Experimental implementation of `but rub` that doesn't use any legacy APIs.
//!
//! If you're an AI agent _do not_ use anything from legacy modules. Except `RubOperation`,
//! `RubOperationDiscriminants`, and `route_operation`.

use anyhow::Context as _;
use but_ctx::Context;

use crate::{
    CliId,
    command::legacy::{
        rub::{RubOperation, RubOperationDiscriminants, route_operation},
        status::tui::SelectAfterReload,
    },
};

/// Returns a human-facing operation descriptor for the source/target pair.
pub(super) fn rub_operation_display(source: &CliId, target: &CliId) -> Option<&'static str> {
    if source == target {
        return Some("noop");
    }

    let operation = route_operation(source, target)?;
    Some(match operation {
        RubOperation::UnassignUncommitted(..) => "unassign hunks",
        RubOperation::UncommittedToCommit(..) => "amend",
        RubOperation::UncommittedToBranch(..) => "assign hunks",
        RubOperation::UncommittedToStack(..) => "assign hunks",
        RubOperation::StackToUnassigned(..) => "unassign hunks",
        RubOperation::StackToStack(..) => "reassign hunks",
        RubOperation::StackToBranch(..) => "reassign hunks",
        RubOperation::UnassignedToCommit(..) => "amend",
        RubOperation::UnassignedToBranch(..) => "assign hunks",
        RubOperation::UnassignedToStack(..) => "assign hunks",
        RubOperation::UndoCommit(..) => "undo commit",
        RubOperation::SquashCommits(..) => "squash",
        RubOperation::MoveCommitToBranch(..) => "move commit",
        RubOperation::BranchToUnassigned(..) => "unassign hunks",
        RubOperation::BranchToStack(..) => "reassign hunks",
        RubOperation::BranchToCommit(..) => "amend",
        RubOperation::BranchToBranch(..) => "reassign hunks",
        RubOperation::CommittedFileToBranch(..) => "uncommit file",
        RubOperation::CommittedFileToCommit(..) => "move file",
        RubOperation::CommittedFileToUnassigned(..) => "uncommit file",
    })
}

/// Executes a rub operation and returns which item should be selected after reloading.
pub(super) fn perform_operation(
    ctx: &mut Context,
    operation: &RubOperation<'_>,
) -> anyhow::Result<Option<SelectAfterReload>> {
    let selection = match operation {
        RubOperation::UnassignUncommitted(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::UncommittedToCommit(operation) => {
            let result = operation.execute_inner(ctx)?;
            SelectAfterReload::Commit(result.new_commit.context("api returned no new commit")?)
        }
        RubOperation::UncommittedToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.name.to_string())
        }
        RubOperation::UncommittedToStack(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::StackToUnassigned(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::StackToStack(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::StackToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.to.to_string())
        }
        RubOperation::UnassignedToCommit(operation) => {
            let result = operation.execute_inner(ctx)?;
            SelectAfterReload::Commit(result.new_commit.context("api returned no new commit")?)
        }
        RubOperation::UnassignedToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.to.to_string())
        }
        RubOperation::UnassignedToStack(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::UndoCommit(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::SquashCommits(operation) => {
            let result = operation.execute_inner(ctx)?;
            SelectAfterReload::Commit(result.new_commit)
        }
        RubOperation::MoveCommitToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.name.to_string())
        }
        RubOperation::BranchToUnassigned(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::BranchToStack(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
        RubOperation::BranchToCommit(operation) => {
            let result = operation.execute_inner(ctx)?;
            result
                .new_commit
                .map(SelectAfterReload::Commit)
                .unwrap_or(SelectAfterReload::Branch(operation.name.to_string()))
        }
        RubOperation::BranchToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.to.to_string())
        }
        RubOperation::CommittedFileToBranch(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Branch(operation.name.to_string())
        }
        RubOperation::CommittedFileToCommit(operation) => {
            let result = operation.execute_inner(ctx)?;
            let destination_to_select = result
                .replaced_commits
                .get(&operation.oid)
                .copied()
                .unwrap_or(operation.oid);
            SelectAfterReload::Commit(destination_to_select)
        }
        RubOperation::CommittedFileToUnassigned(operation) => {
            operation.execute_inner(ctx)?;
            SelectAfterReload::Unassigned
        }
    };

    Ok(Some(selection))
}

<<<<<<< New base: Make stack_id derived: remove from DB, compute from branch_ref_bytes
||||||| Common ancestor
/// Executes `UnassignUncommitted` by writing explicit unassigned requests via the rub API.
fn execute_unassign_uncommitted(
    ctx: &mut Context,
    operation: &UnassignUncommittedOperation<'_>,
) -> anyhow::Result<()> {
    let requests =
        assignment_requests_for_selected_hunks(operation.hunk_assignments.iter().copied(), None);
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `UncommittedToCommit` and returns the exact commit-amend API result.
fn execute_uncommitted_to_commit(
    ctx: &mut Context,
    operation: &UncommittedToCommitOperation<'_>,
) -> anyhow::Result<CommitCreateResult> {
    let changes = operation
        .hunk_assignments
        .iter()
        .copied()
        .cloned()
        .map(DiffSpec::from)
        .collect::<Vec<_>>();
    let changes = but_workspace::flatten_diff_specs(changes);
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `UncommittedToBranch` by assigning selected hunks to the target branch stack.
fn execute_uncommitted_to_branch(
    ctx: &mut Context,
    operation: &UncommittedToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let requests = assignment_requests_for_selected_hunks(
        operation.hunk_assignments.iter().copied(),
        stack_id,
    );
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `UncommittedToStack` by assigning selected hunks to the target stack.
fn execute_uncommitted_to_stack(
    ctx: &mut Context,
    operation: &UncommittedToStackOperation<'_>,
) -> anyhow::Result<()> {
    let requests = assignment_requests_for_selected_hunks(
        operation.hunk_assignments.iter().copied(),
        Some(operation.stack_id),
    );
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `StackToUnassigned` by reassigning all hunks from the source stack into unassigned.
fn execute_stack_to_unassigned(
    ctx: &mut Context,
    operation: &StackToUnassignedOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, Some(operation.stack_id), None)
}

/// Executes `StackToStack` by reassigning all hunks from one stack to another.
fn execute_stack_to_stack(
    ctx: &mut Context,
    operation: &StackToStackOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, Some(operation.from), Some(operation.to))
}

/// Executes `StackToBranch` by reassigning all hunks from the source stack to the target branch stack.
fn execute_stack_to_branch(
    ctx: &mut Context,
    operation: &StackToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, Some(operation.from), target_stack_id)
}

/// Executes `UnassignedToCommit` and returns the exact commit-amend API result.
fn execute_unassigned_to_commit(
    ctx: &mut Context,
    operation: &UnassignedToCommitOperation,
) -> anyhow::Result<CommitCreateResult> {
    let changes = changes_for_stack_assignment(ctx, None)?;
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `UnassignedToBranch` by assigning unassigned hunks to the target branch stack.
fn execute_unassigned_to_branch(
    ctx: &mut Context,
    operation: &UnassignedToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, None, target_stack_id)
}

/// Executes `UnassignedToStack` by assigning unassigned hunks to the target stack.
fn execute_unassigned_to_stack(
    ctx: &mut Context,
    operation: &UnassignedToStackOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, None, Some(operation.to))
}

/// Executes `UndoCommit` by uncommitting all changes from the selected commit.
fn execute_undo_commit(
    ctx: &mut Context,
    operation: &UndoCommitOperation,
) -> anyhow::Result<CommitUndoResult> {
    let UndoCommitOperation { oid } = operation;
    but_api::commit::undo::commit_undo(ctx, *oid)
}

/// Executes `SquashCommits` by squashing source into target.
fn execute_squash_commits(
    ctx: &mut Context,
    operation: &SquashCommitsOperation,
) -> anyhow::Result<CommitSquashResult> {
    let SquashCommitsOperation {
        source,
        destination,
    } = operation;
    but_api::commit::squash::commit_squash(ctx, *source, *destination)
}

/// Executes `MoveCommitToBranch` and returns the exact commit-move API result.
fn execute_move_commit_to_branch(
    ctx: &mut Context,
    operation: &MoveCommitToBranchOperation<'_>,
) -> anyhow::Result<CommitMoveResult> {
    let target_full_name = FullName::try_from(format!("refs/heads/{}", operation.name))?;
    but_api::commit::move_commit::commit_move(
        ctx,
        operation.oid,
        RelativeTo::Reference(target_full_name),
        InsertSide::Below,
    )
}

/// Executes `BranchToUnassigned` by moving all branch-assigned hunks into unassigned.
fn execute_branch_to_unassigned(
    ctx: &mut Context,
    operation: &BranchToUnassignedOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, None)
}

/// Executes `BranchToStack` by moving all branch-assigned hunks into the target stack.
fn execute_branch_to_stack(
    ctx: &mut Context,
    operation: &BranchToStackOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, Some(operation.to))
}

/// Executes `BranchToCommit` and returns the exact commit-amend API result.
///
/// When the source branch is not associated with a stack, this amends currently
/// unassigned hunks to match legacy `but rub` behavior.
fn execute_branch_to_commit(
    ctx: &mut Context,
    operation: &BranchToCommitOperation<'_>,
) -> anyhow::Result<CommitCreateResult> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let changes = changes_for_stack_assignment(ctx, stack_id)?;
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `BranchToBranch` by reassigning all hunks from one branch stack to another.
fn execute_branch_to_branch(
    ctx: &mut Context,
    operation: &BranchToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, target_stack_id)
}

/// Executes `CommittedFileToBranch` and returns the exact uncommit API result.
///
/// When the target branch is not associated with a stack, this uncommits file
/// changes into unassigned to match legacy `but rub` behavior.
fn execute_committed_file_to_branch(
    ctx: &mut Context,
    operation: &CommittedFileToBranchOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::uncommit::commit_uncommit_changes(
        ctx,
        operation.commit_oid,
        relevant_changes,
        stack_id,
    )
}

/// Executes `CommittedFileToCommit` and returns the exact move-changes API result.
fn execute_committed_file_to_commit(
    ctx: &mut Context,
    operation: &CommittedFileToCommitOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::move_changes::commit_move_changes_between(
        ctx,
        operation.commit_oid,
        operation.oid,
        relevant_changes,
    )
}

/// Executes `CommittedFileToUnassigned` and returns the exact uncommit API result.
fn execute_committed_file_to_unassigned(
    ctx: &mut Context,
    operation: &CommittedFileToUnassignedOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::uncommit::commit_uncommit_changes(
        ctx,
        operation.commit_oid,
        relevant_changes,
        None,
    )
}

/// Builds assignment requests for selected hunks and assigns them to `target_stack_id`.
fn assignment_requests_for_selected_hunks<'a>(
    hunks: impl Iterator<Item = &'a but_hunk_assignment::HunkAssignment>,
    target_stack_id: Option<StackId>,
) -> Vec<HunkAssignmentRequest> {
    hunks
        .map(|assignment| HunkAssignmentRequest {
            hunk_header: assignment.hunk_header,
            path_bytes: assignment.path_bytes.to_owned(),
            stack_id: target_stack_id,
            branch_ref_bytes: None,
        })
        .collect()
}

/// Reassigns all current worktree assignments from `source_stack_id` to `target_stack_id`.
fn reassign_all_from_stack_to_stack(
    ctx: &mut Context,
    source_stack_id: Option<StackId>,
    target_stack_id: Option<StackId>,
) -> anyhow::Result<()> {
    let requests = but_api::diff::changes_in_worktree(ctx)?
        .assignments
        .into_iter()
        .filter(|assignment| assignment.stack_id == source_stack_id)
        .map(|assignment| HunkAssignmentRequest {
            hunk_header: assignment.hunk_header,
            path_bytes: assignment.path_bytes,
            stack_id: target_stack_id,
            branch_ref_bytes: None,
        })
        .collect::<Vec<_>>();

    but_api::diff::assign_hunk(ctx, requests)
}

/// Collects worktree diff specs that are currently assigned to `stack_id`.
fn changes_for_stack_assignment(
    ctx: &mut Context,
    stack_id: Option<StackId>,
) -> anyhow::Result<Vec<DiffSpec>> {
    let changes = but_api::diff::changes_in_worktree(ctx)?
        .assignments
        .into_iter()
        .filter(|assignment| assignment.stack_id == stack_id)
        .map(DiffSpec::from)
        .collect();
    Ok(but_workspace::flatten_diff_specs(changes))
}

/// Resolves a branch name into its workspace stack id, if any.
fn stack_id_for_branch_name(
    ctx: &mut Context,
    branch_name: &str,
) -> anyhow::Result<Option<StackId>> {
    let target_branch_full_name = FullName::try_from(format!("refs/heads/{branch_name}"))?;
    let (_guard, _repo, ws, _db) = ctx.workspace_and_db()?;
    Ok(ws
        .find_segment_and_stack_by_refname(target_branch_full_name.as_ref())
        .and_then(|(stack, _segment)| stack.id))
}

/// Computes diff specs for changes to `path` in `commit_oid` relative to its first parent.
fn file_changes_from_commit(
    ctx: &Context,
    commit_oid: gix::ObjectId,
    path: &bstr::BStr,
) -> anyhow::Result<Vec<DiffSpec>> {
    let repo = ctx.repo.get()?;
    let source_commit = repo.find_commit(commit_oid)?;
    let source_commit_parent_id = source_commit.parent_ids().next().context("no parents")?;

    let tree_changes = tree_changes(&repo, Some(source_commit_parent_id.detach()), commit_oid)?;
    Ok(tree_changes
        .into_iter()
        .filter(|tc| tc.path == path)
        .map(DiffSpec::from)
        .collect::<Vec<_>>())
}

=======
/// Executes `UnassignUncommitted` by writing explicit unassigned requests via the rub API.
fn execute_unassign_uncommitted(
    ctx: &mut Context,
    operation: &UnassignUncommittedOperation<'_>,
) -> anyhow::Result<()> {
    let requests =
        assignment_requests_for_selected_hunks(operation.hunk_assignments.iter().copied(), None);
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `UncommittedToCommit` and returns the exact commit-amend API result.
fn execute_uncommitted_to_commit(
    ctx: &mut Context,
    operation: &UncommittedToCommitOperation<'_>,
) -> anyhow::Result<CommitCreateResult> {
    let changes = operation
        .hunk_assignments
        .iter()
        .copied()
        .cloned()
        .map(DiffSpec::from)
        .collect::<Vec<_>>();
    let changes = but_workspace::flatten_diff_specs(changes);
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `UncommittedToBranch` by assigning selected hunks to the target branch stack.
fn execute_uncommitted_to_branch(
    ctx: &mut Context,
    operation: &UncommittedToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let requests = assignment_requests_for_selected_hunks(
        operation.hunk_assignments.iter().copied(),
        stack_id,
    );
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `UncommittedToStack` by assigning selected hunks to the target stack.
fn execute_uncommitted_to_stack(
    ctx: &mut Context,
    operation: &UncommittedToStackOperation<'_>,
) -> anyhow::Result<()> {
    let requests = assignment_requests_for_selected_hunks(
        operation.hunk_assignments.iter().copied(),
        Some(operation.stack_id),
    );
    but_api::diff::assign_hunk(ctx, requests)
}

/// Executes `StackToUnassigned` by reassigning all hunks from the source stack into unassigned.
fn execute_stack_to_unassigned(
    ctx: &mut Context,
    operation: &StackToUnassignedOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, Some(operation.stack_id), None)
}

/// Executes `StackToStack` by reassigning all hunks from one stack to another.
fn execute_stack_to_stack(
    ctx: &mut Context,
    operation: &StackToStackOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, Some(operation.from), Some(operation.to))
}

/// Executes `StackToBranch` by reassigning all hunks from the source stack to the target branch stack.
fn execute_stack_to_branch(
    ctx: &mut Context,
    operation: &StackToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, Some(operation.from), target_stack_id)
}

/// Executes `UnassignedToCommit` and returns the exact commit-amend API result.
fn execute_unassigned_to_commit(
    ctx: &mut Context,
    operation: &UnassignedToCommitOperation,
) -> anyhow::Result<CommitCreateResult> {
    let changes = changes_for_stack_assignment(ctx, None)?;
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `UnassignedToBranch` by assigning unassigned hunks to the target branch stack.
fn execute_unassigned_to_branch(
    ctx: &mut Context,
    operation: &UnassignedToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, None, target_stack_id)
}

/// Executes `UnassignedToStack` by assigning unassigned hunks to the target stack.
fn execute_unassigned_to_stack(
    ctx: &mut Context,
    operation: &UnassignedToStackOperation,
) -> anyhow::Result<()> {
    reassign_all_from_stack_to_stack(ctx, None, Some(operation.to))
}

/// Executes `UndoCommit` by uncommitting all changes from the selected commit.
fn execute_undo_commit(
    ctx: &mut Context,
    operation: &UndoCommitOperation,
) -> anyhow::Result<CommitUndoResult> {
    let UndoCommitOperation { oid } = operation;
    but_api::commit::undo::commit_undo(ctx, *oid)
}

/// Executes `SquashCommits` by squashing source into target.
fn execute_squash_commits(
    ctx: &mut Context,
    operation: &SquashCommitsOperation,
) -> anyhow::Result<CommitSquashResult> {
    let SquashCommitsOperation {
        source,
        destination,
    } = operation;
    but_api::commit::squash::commit_squash(ctx, *source, *destination)
}

/// Executes `MoveCommitToBranch` and returns the exact commit-move API result.
fn execute_move_commit_to_branch(
    ctx: &mut Context,
    operation: &MoveCommitToBranchOperation<'_>,
) -> anyhow::Result<CommitMoveResult> {
    let target_full_name = FullName::try_from(format!("refs/heads/{}", operation.name))?;
    but_api::commit::move_commit::commit_move(
        ctx,
        operation.oid,
        RelativeTo::Reference(target_full_name),
        InsertSide::Below,
    )
}

/// Executes `BranchToUnassigned` by moving all branch-assigned hunks into unassigned.
fn execute_branch_to_unassigned(
    ctx: &mut Context,
    operation: &BranchToUnassignedOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, None)
}

/// Executes `BranchToStack` by moving all branch-assigned hunks into the target stack.
fn execute_branch_to_stack(
    ctx: &mut Context,
    operation: &BranchToStackOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, Some(operation.to))
}

/// Executes `BranchToCommit` and returns the exact commit-amend API result.
///
/// When the source branch is not associated with a stack, this amends currently
/// unassigned hunks to match legacy `but rub` behavior.
fn execute_branch_to_commit(
    ctx: &mut Context,
    operation: &BranchToCommitOperation<'_>,
) -> anyhow::Result<CommitCreateResult> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let changes = changes_for_stack_assignment(ctx, stack_id)?;
    but_api::commit::amend::commit_amend(ctx, operation.oid, changes)
}

/// Executes `BranchToBranch` by reassigning all hunks from one branch stack to another.
fn execute_branch_to_branch(
    ctx: &mut Context,
    operation: &BranchToBranchOperation<'_>,
) -> anyhow::Result<()> {
    let source_stack_id = stack_id_for_branch_name(ctx, operation.from)?;
    let target_stack_id = stack_id_for_branch_name(ctx, operation.to)?;
    reassign_all_from_stack_to_stack(ctx, source_stack_id, target_stack_id)
}

/// Executes `CommittedFileToBranch` and returns the exact uncommit API result.
///
/// When the target branch is not associated with a stack, this uncommits file
/// changes into unassigned to match legacy `but rub` behavior.
fn execute_committed_file_to_branch(
    ctx: &mut Context,
    operation: &CommittedFileToBranchOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let stack_id = stack_id_for_branch_name(ctx, operation.name)?;
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::uncommit::commit_uncommit_changes(
        ctx,
        operation.commit_oid,
        relevant_changes,
        stack_id,
    )
}

/// Executes `CommittedFileToCommit` and returns the exact move-changes API result.
fn execute_committed_file_to_commit(
    ctx: &mut Context,
    operation: &CommittedFileToCommitOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::move_changes::commit_move_changes_between(
        ctx,
        operation.commit_oid,
        operation.oid,
        relevant_changes,
    )
}

/// Executes `CommittedFileToUnassigned` and returns the exact uncommit API result.
fn execute_committed_file_to_unassigned(
    ctx: &mut Context,
    operation: &CommittedFileToUnassignedOperation<'_>,
) -> anyhow::Result<MoveChangesResult> {
    let relevant_changes = file_changes_from_commit(ctx, operation.commit_oid, operation.path)?;
    but_api::commit::uncommit::commit_uncommit_changes(
        ctx,
        operation.commit_oid,
        relevant_changes,
        None,
    )
}

/// Builds assignment requests for selected hunks and assigns them to `target_stack_id`.
fn assignment_requests_for_selected_hunks<'a>(
    hunks: impl Iterator<Item = &'a but_hunk_assignment::HunkAssignment>,
    target_stack_id: Option<StackId>,
) -> Vec<HunkAssignmentRequest> {
    hunks
        .map(|assignment| HunkAssignmentRequest {
            hunk_header: assignment.hunk_header,
            path_bytes: assignment.path_bytes.to_owned(),
            stack_id: target_stack_id,
            // Clear branch_ref so it gets resolved from the target stack
            branch_ref_bytes: None,
        })
        .collect()
}

/// Reassigns all current worktree assignments from `source_stack_id` to `target_stack_id`.
fn reassign_all_from_stack_to_stack(
    ctx: &mut Context,
    source_stack_id: Option<StackId>,
    target_stack_id: Option<StackId>,
) -> anyhow::Result<()> {
    let requests = but_api::diff::changes_in_worktree(ctx)?
        .assignments
        .into_iter()
        .filter(|assignment| assignment.stack_id == source_stack_id)
        .map(|assignment| HunkAssignmentRequest {
            hunk_header: assignment.hunk_header,
            path_bytes: assignment.path_bytes,
            stack_id: target_stack_id,
            // Clear branch_ref so it gets resolved from the target stack
            branch_ref_bytes: None,
        })
        .collect::<Vec<_>>();

    but_api::diff::assign_hunk(ctx, requests)
}

/// Collects worktree diff specs that are currently assigned to `stack_id`.
fn changes_for_stack_assignment(
    ctx: &mut Context,
    stack_id: Option<StackId>,
) -> anyhow::Result<Vec<DiffSpec>> {
    let changes = but_api::diff::changes_in_worktree(ctx)?
        .assignments
        .into_iter()
        .filter(|assignment| assignment.stack_id == stack_id)
        .map(DiffSpec::from)
        .collect();
    Ok(but_workspace::flatten_diff_specs(changes))
}

/// Resolves a branch name into its workspace stack id, if any.
fn stack_id_for_branch_name(
    ctx: &mut Context,
    branch_name: &str,
) -> anyhow::Result<Option<StackId>> {
    let target_branch_full_name = FullName::try_from(format!("refs/heads/{branch_name}"))?;
    let (_guard, _repo, ws, _db) = ctx.workspace_and_db()?;
    Ok(ws
        .find_segment_and_stack_by_refname(target_branch_full_name.as_ref())
        .and_then(|(stack, _segment)| stack.id))
}

/// Computes diff specs for changes to `path` in `commit_oid` relative to its first parent.
fn file_changes_from_commit(
    ctx: &Context,
    commit_oid: gix::ObjectId,
    path: &bstr::BStr,
) -> anyhow::Result<Vec<DiffSpec>> {
    let repo = ctx.repo.get()?;
    let source_commit = repo.find_commit(commit_oid)?;
    let source_commit_parent_id = source_commit.parent_ids().next().context("no parents")?;

    let tree_changes = tree_changes(&repo, Some(source_commit_parent_id.detach()), commit_oid)?;
    Ok(tree_changes
        .into_iter()
        .filter(|tc| tc.path == path)
        .map(DiffSpec::from)
        .collect::<Vec<_>>())
}

>>>>>>> Current commit: Make stack_id derived: remove from DB, compute from branch_ref_bytes
/// Error raised when a routed operation has no implementation in this rub-api module.
#[derive(Debug)]
pub(super) struct OperationNotSupported(RubOperationDiscriminants);

impl OperationNotSupported {
    /// Creates an unsupported-operation error from a routed operation value.
    pub(super) fn new(operation: &RubOperation<'_>) -> Self {
        OperationNotSupported(RubOperationDiscriminants::from(operation))
    }
}

impl std::fmt::Display for OperationNotSupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} is not supported", self.0)
    }
}

impl std::error::Error for OperationNotSupported {}

#[cfg(test)]
mod tests {
    use bstr::BString;

    use super::rub_operation_display;
    use crate::CliId;

    /// Converts a hex object id into `gix::ObjectId` for test setup.
    fn commit_id(hex: &str) -> gix::ObjectId {
        gix::ObjectId::from_hex(hex.as_bytes()).unwrap()
    }

    #[test]
    fn branch_to_commit_is_supported_when_source_branch_has_no_stack() {
        let source = CliId::Branch {
            name: "main".into(),
            id: "b0".into(),
            stack_id: None,
        };
        let target = CliId::Commit {
            commit_id: commit_id("1111111111111111111111111111111111111111"),
            id: "c0".into(),
        };

        assert_eq!(rub_operation_display(&source, &target).unwrap(), "amend");
    }

    #[test]
    fn committed_file_to_branch_is_supported_when_target_branch_has_no_stack() {
        let source = CliId::CommittedFile {
            commit_id: commit_id("1111111111111111111111111111111111111111"),
            path: BString::from("file.txt"),
            id: "f0".into(),
        };
        let target = CliId::Branch {
            name: "main".into(),
            id: "b0".into(),
            stack_id: None,
        };

        assert_eq!(
            rub_operation_display(&source, &target).unwrap(),
            "uncommit file"
        );
    }
}
