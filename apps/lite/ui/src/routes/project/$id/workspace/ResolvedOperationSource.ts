import {
	changesInWorktreeQueryOptions,
	commitDetailsWithLineStatsQueryOptions,
} from "#ui/api/queries.ts";
import {
	assignHunkOperation,
	commitAmendOperation,
	commitCreateFromCommittedChangesOperation,
	commitCreateOperation,
	commitMoveChangesBetweenOperation,
	commitMoveOperation,
	commitSquashOperation,
	commitUncommitChangesOperation,
	commitUncommitOperation,
	moveBranchOperation,
	tearOffBranchOperation,
	type Operation,
} from "#ui/Operation.ts";
import { createDiffSpec } from "#ui/domain/DiffSpec.ts";
import { changesSectionFileParent, type FileParent } from "#ui/domain/FileParent.ts";
import { useQueryClient } from "@tanstack/react-query";
import {
	CommitDetails,
	HunkAssignmentRequest,
	InsertSide,
	WorktreeChanges,
	type HunkAssignment,
	type HunkHeader,
	type TreeChange,
} from "@gitbutler/but-sdk";
import { Match } from "effect";
import { decodeRefName, getAssignmentsByPath } from "../shared";
import { type OperationSource } from "./OperationSource.ts";

type TreeChangeWithHunkHeaders = {
	change: TreeChange;
	hunkHeaders: Array<HunkHeader>;
};

/** @public */
export type CommitResolvedOperationSource = { commitId: string };
/** @public */
export type SegmentResolvedOperationSource = { branchRef: Array<number> | null };
/** @public */
export type TreeChangesResolvedOperationSource = {
	parent: FileParent;
	changes: Array<TreeChangeWithHunkHeaders>;
};

/**
 * The source of an operation in a form that can be sent to the backend.
 */
export type ResolvedOperationSource =
	| { _tag: "BaseCommit" }
	| ({ _tag: "Commit" } & CommitResolvedOperationSource)
	| ({ _tag: "Segment" } & SegmentResolvedOperationSource)
	| ({ _tag: "TreeChanges" } & TreeChangesResolvedOperationSource);

/** @public */
export const baseCommitResolvedOperationSource: ResolvedOperationSource = {
	_tag: "BaseCommit",
};

/** @public */
export const commitResolvedOperationSource = ({
	commitId,
}: CommitResolvedOperationSource): ResolvedOperationSource => ({
	_tag: "Commit",
	commitId,
});

/** @public */
export const segmentResolvedOperationSource = ({
	branchRef,
}: SegmentResolvedOperationSource): ResolvedOperationSource => ({
	_tag: "Segment",
	branchRef,
});

/** @public */
export const treeChangesResolvedOperationSource = ({
	parent,
	changes,
}: TreeChangesResolvedOperationSource): ResolvedOperationSource => ({
	_tag: "TreeChanges",
	parent,
	changes,
});

const hunkHeadersForAssignments = (
	assignments: Array<HunkAssignment> | undefined,
): Array<HunkHeader> =>
	assignments
		? assignments.flatMap((assignment) =>
				assignment.hunkHeader != null ? [assignment.hunkHeader] : [],
			)
		: [];

const resolveOperationSource = ({
	operationSource,
	worktreeChanges,
	getCommitDetails,
}: {
	operationSource: OperationSource;
	worktreeChanges: WorktreeChanges | undefined;
	getCommitDetails: (commitId: string) => CommitDetails | undefined;
}) =>
	Match.value(operationSource).pipe(
		Match.tagsExhaustive({
			Segment: ({ branchRef }) => segmentResolvedOperationSource({ branchRef }),
			BaseCommit: () => baseCommitResolvedOperationSource,
			Commit: ({ commitId }) => commitResolvedOperationSource({ commitId }),
			ChangesSection: ({ stackId }) => {
				if (!worktreeChanges) return null;

				const assignmentsByPath = getAssignmentsByPath(worktreeChanges.assignments, stackId);
				const changes = worktreeChanges.changes.flatMap(
					(change): Array<TreeChangeWithHunkHeaders> => {
						const assignments = assignmentsByPath.get(change.path);
						if (!assignments) return [];

						return [
							{
								change,
								hunkHeaders: hunkHeadersForAssignments(assignments),
							},
						];
					},
				);

				return treeChangesResolvedOperationSource({
					parent: changesSectionFileParent({ stackId }),
					changes,
				});
			},
			File: ({ parent, path }) => {
				const change = Match.value(parent).pipe(
					Match.tagsExhaustive({
						ChangesSection: () => {
							if (!worktreeChanges) return null;

							return worktreeChanges.changes.find((candidate) => candidate.path === path) ?? null;
						},
						Commit: ({ commitId }) => {
							const commitDetails = getCommitDetails(commitId);
							if (!commitDetails) return null;

							return commitDetails.changes.find((candidate) => candidate.path === path) ?? null;
						},
					}),
				);

				if (!change) return null;

				const hunkHeaders = Match.value(parent).pipe(
					Match.tagsExhaustive({
						ChangesSection: ({ stackId }) => {
							if (!worktreeChanges) return [];

							return hunkHeadersForAssignments(
								getAssignmentsByPath(worktreeChanges.assignments, stackId).get(path),
							);
						},
						Commit: () => [],
					}),
				);

				return treeChangesResolvedOperationSource({
					parent,
					changes: [{ change, hunkHeaders }],
				});
			},
			Hunk: ({ parent, path, hunkHeader }) => {
				const change = Match.value(parent).pipe(
					Match.tagsExhaustive({
						ChangesSection: () => {
							if (!worktreeChanges) return null;

							return worktreeChanges.changes.find((candidate) => candidate.path === path) ?? null;
						},
						Commit: ({ commitId }) => {
							const commitDetails = getCommitDetails(commitId);
							if (!commitDetails) return null;

							return commitDetails.changes.find((candidate) => candidate.path === path) ?? null;
						},
					}),
				);

				if (!change) return null;

				return treeChangesResolvedOperationSource({
					parent,
					changes: [{ change, hunkHeaders: [hunkHeader] }],
				});
			},
		}),
	);

export const useResolveOperationSource = (projectId: string) => {
	const queryClient = useQueryClient();

	return (operationSource: OperationSource) =>
		resolveOperationSource({
			operationSource,
			worktreeChanges: queryClient.getQueryData(changesInWorktreeQueryOptions(projectId).queryKey),
			getCommitDetails: (commitId) =>
				queryClient.getQueryData(
					commitDetailsWithLineStatsQueryOptions({ projectId, commitId }).queryKey,
				),
		});
};

/**
 * | SOURCE ↓ / TARGET →    | Changes  | Commit |
 * | ---------------------- | -------- | ------ |
 * | File/hunk from changes | Assign   | Amend  |
 * | File/hunk from commit  | Uncommit | Amend  |
 * | Commit                 | Uncommit | Squash |
 *
 * Note this is currently different from the CLI's definition of "rubbing",
 * which also includes move operations.
 * https://linear.app/gitbutler/issue/GB-1160/what-should-rubbing-a-branch-into-another-branch-do#comment-db2abdb7
 */
export const getCombineOperation = ({
	resolvedOperationSource,
	target,
}: {
	resolvedOperationSource: ResolvedOperationSource;
	target: FileParent;
}): Operation | null =>
	Match.value(resolvedOperationSource).pipe(
		Match.tagsExhaustive({
			Segment: () => null,
			BaseCommit: () => null,
			Commit: ({ commitId: sourceCommitId }) =>
				Match.value(target).pipe(
					Match.tagsExhaustive({
						ChangesSection: ({ stackId }) =>
							commitUncommitOperation({
								commitId: sourceCommitId,
								assignTo: stackId,
							}),
						Commit: ({ commitId: destinationCommitId }) =>
							commitSquashOperation({
								sourceCommitId,
								destinationCommitId,
								dryRun: false,
							}),
					}),
				),
			TreeChanges: ({ parent, changes: sourceChanges }) => {
				const changes = sourceChanges.map(({ change, hunkHeaders }) =>
					createDiffSpec(change, hunkHeaders),
				);

				return Match.value(parent).pipe(
					Match.tagsExhaustive({
						ChangesSection: () =>
							Match.value(target).pipe(
								Match.tagsExhaustive({
									ChangesSection: ({ stackId: targetStackId }) =>
										assignHunkOperation({
											assignments: sourceChanges.flatMap(({ change, hunkHeaders }) =>
												hunkHeaders.map(
													(hunkHeader): HunkAssignmentRequest => ({
														pathBytes: change.pathBytes,
														hunkHeader,
														stackId: targetStackId,
														branchRefBytes: null,
													}),
												),
											),
										}),
									Commit: ({ commitId }) =>
										commitAmendOperation({
											commitId,
											changes,
											dryRun: false,
										}),
								}),
							),
						Commit: ({ commitId: sourceCommitId }) =>
							Match.value(target).pipe(
								Match.tagsExhaustive({
									ChangesSection: ({ stackId }) =>
										commitUncommitChangesOperation({
											commitId: sourceCommitId,
											assignTo: stackId,
											changes,
											dryRun: false,
										}),
									Commit: ({ commitId: destinationCommitId }) =>
										commitMoveChangesBetweenOperation({
											sourceCommitId,
											destinationCommitId,
											changes,
											dryRun: false,
										}),
								}),
							),
					}),
				);
			},
		}),
	);

export const getCommitTargetMoveOperation = ({
	resolvedOperationSource,
	commitId,
	side,
}: {
	resolvedOperationSource: ResolvedOperationSource;
	commitId: string;
	side: InsertSide;
}) =>
	Match.value(resolvedOperationSource).pipe(
		Match.tags({
			Commit: ({ commitId: subjectCommitId }) =>
				commitMoveOperation({
					subjectCommitId,
					relativeTo: { type: "commit", subject: commitId },
					side,
					dryRun: false,
				}),
			TreeChanges: ({ parent, changes: sourceChanges }) => {
				const changes = sourceChanges.map(({ change, hunkHeaders }) =>
					createDiffSpec(change, hunkHeaders),
				);

				return Match.value(parent).pipe(
					Match.tags({
						ChangesSection: () =>
							commitCreateOperation({
								relativeTo: { type: "commit", subject: commitId },
								side,
								changes,
								message: "",
								dryRun: false,
							}),
						Commit: ({ commitId: sourceCommitId }) =>
							commitCreateFromCommittedChangesOperation({
								sourceCommitId,
								relativeTo: { type: "commit", subject: commitId },
								side,
								changes,
								dryRun: false,
							}),
					}),
					Match.exhaustive,
				);
			},
		}),
		Match.orElse(() => null),
	);

export const getBranchTargetOperation = ({
	resolvedOperationSource,
	branchRef,
}: {
	resolvedOperationSource: ResolvedOperationSource;
	branchRef: Array<number>;
}): Operation | null =>
	Match.value(resolvedOperationSource).pipe(
		Match.tags({
			Segment: (source) => {
				if (source.branchRef === null) return null;
				return moveBranchOperation({
					subjectBranch: decodeRefName(source.branchRef),
					targetBranch: decodeRefName(branchRef),
					dryRun: false,
				});
			},
			Commit: ({ commitId }) =>
				commitMoveOperation({
					subjectCommitId: commitId,
					relativeTo: {
						type: "referenceBytes",
						subject: branchRef,
					},
					side: "below",
					dryRun: false,
				}),
			TreeChanges: (source) => {
				const changes = source.changes.map(({ change, hunkHeaders }) =>
					createDiffSpec(change, hunkHeaders),
				);

				return Match.value(source.parent).pipe(
					Match.tagsExhaustive({
						ChangesSection: () =>
							commitCreateOperation({
								relativeTo: { type: "referenceBytes", subject: branchRef },
								side: "below",
								changes,
								message: "",
								dryRun: false,
							}),
						Commit: ({ commitId: sourceCommitId }) =>
							commitCreateFromCommittedChangesOperation({
								sourceCommitId,
								relativeTo: { type: "referenceBytes", subject: branchRef },
								side: "below",
								changes,
								dryRun: false,
							}),
					}),
				);
			},
		}),
		Match.orElse(() => null),
	);

export const getTearOffBranchTargetOperation = (
	resolvedOperationSource: ResolvedOperationSource,
): Operation | null => {
	if (resolvedOperationSource._tag !== "Segment") return null;
	if (resolvedOperationSource.branchRef === null) return null;

	return tearOffBranchOperation({
		subjectBranch: decodeRefName(resolvedOperationSource.branchRef),
		dryRun: false,
	});
};
