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
