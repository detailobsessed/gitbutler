import { Toast } from "@base-ui/react";
import { useMutation } from "@tanstack/react-query";
import { Match } from "effect";
import {
	type AssignHunkParams,
	type CommitAmendParams,
	type CommitCreateParams,
	type CommitInsertBlankParams,
	type CommitMoveParams,
	type CommitMoveChangesBetweenParams,
	type CommitUncommitChangesParams,
	type MoveBranchParams,
	type TearOffBranchParams,
	CommitSquashParams,
} from "#electron/ipc.ts";
import { rejectedChangesToastOptions } from "#ui/components/RejectedChanges.tsx";
import {
	assignHunkMutationOptions,
	commitAmendMutationOptions,
	commitCreateMutationOptions,
	commitInsertBlankMutationOptions,
	commitMoveMutationOptions,
	commitMoveChangesBetweenMutationOptions,
	commitSquashMutationOptions,
	commitUncommitChangesMutationOptions,
	commitUncommitMutationOptions,
	moveBranchMutationOptions,
	tearOffBranchMutationOptions,
	CommitUncommitParams,
} from "#ui/api/mutations.ts";
import { InsertSide } from "@gitbutler/but-sdk";

/** @public */
export type AssignHunkOperation = Omit<AssignHunkParams, "projectId">;
/** @public */
export type CommitAmendOperation = Omit<CommitAmendParams, "projectId">;
/** @public */
export type CommitCreateOperation = Omit<CommitCreateParams, "projectId">;
/** @public */
export type CommitCreateFromCommittedChangesOperation = Omit<CommitInsertBlankParams, "projectId"> &
	Pick<CommitMoveChangesBetweenParams, "changes" | "sourceCommitId">;
/** @public */
export type CommitMoveOperation = Omit<CommitMoveParams, "projectId">;
/** @public */
export type CommitMoveChangesBetweenOperation = Omit<CommitMoveChangesBetweenParams, "projectId">;
/** @public */
export type CommitSquashOperation = Omit<CommitSquashParams, "projectId">;
/** @public */
export type CommitUncommitOperation = Omit<CommitUncommitParams, "projectId">;
/** @public */
export type CommitUncommitChangesOperation = Omit<CommitUncommitChangesParams, "projectId">;
/** @public */
export type MoveBranchOperation = Omit<MoveBranchParams, "projectId">;
/** @public */
export type TearOffBranchOperation = Omit<TearOffBranchParams, "projectId">;

export type Operation =
	| ({ _tag: "AssignHunk" } & AssignHunkOperation)
	| ({ _tag: "CommitAmend" } & CommitAmendOperation)
	| ({ _tag: "CommitCreate" } & CommitCreateOperation)
	| ({ _tag: "CommitCreateFromCommittedChanges" } & CommitCreateFromCommittedChangesOperation)
	| ({ _tag: "CommitMove" } & CommitMoveOperation)
	| ({ _tag: "CommitMoveChangesBetween" } & CommitMoveChangesBetweenOperation)
	| ({ _tag: "CommitSquash" } & CommitSquashOperation)
	| ({ _tag: "CommitUncommit" } & CommitUncommitOperation)
	| ({ _tag: "CommitUncommitChanges" } & CommitUncommitChangesOperation)
	| ({ _tag: "MoveBranch" } & MoveBranchOperation)
	| ({ _tag: "TearOffBranch" } & TearOffBranchOperation);

/** @public */
export const assignHunkOperation = (operation: AssignHunkOperation): Operation => ({
	_tag: "AssignHunk",
	...operation,
});

/** @public */
export const commitAmendOperation = (operation: CommitAmendOperation): Operation => ({
	_tag: "CommitAmend",
	...operation,
});

/** @public */
export const commitCreateOperation = (operation: CommitCreateOperation): Operation => ({
	_tag: "CommitCreate",
	...operation,
});

/** @public */
export const commitCreateFromCommittedChangesOperation = (
	operation: CommitCreateFromCommittedChangesOperation,
): Operation => ({
	_tag: "CommitCreateFromCommittedChanges",
	...operation,
});

/** @public */
export const commitMoveOperation = (operation: CommitMoveOperation): Operation => ({
	_tag: "CommitMove",
	...operation,
});

/** @public */
export const commitMoveChangesBetweenOperation = (
	operation: CommitMoveChangesBetweenOperation,
): Operation => ({
	_tag: "CommitMoveChangesBetween",
	...operation,
});

/** @public */
export const commitSquashOperation = (operation: CommitSquashOperation): Operation => ({
	_tag: "CommitSquash",
	...operation,
});

/** @public */
export const commitUncommitOperation = (operation: CommitUncommitOperation): Operation => ({
	_tag: "CommitUncommit",
	...operation,
});

/** @public */
export const commitUncommitChangesOperation = (
	operation: CommitUncommitChangesOperation,
): Operation => ({
	_tag: "CommitUncommitChanges",
	...operation,
});

/** @public */
export const moveBranchOperation = (operation: MoveBranchOperation): Operation => ({
	_tag: "MoveBranch",
	...operation,
});

/** @public */
export const tearOffBranchOperation = (operation: TearOffBranchOperation): Operation => ({
	_tag: "TearOffBranch",
	...operation,
});

export const getInsertionSide = (operation: Operation): InsertSide | null =>
	Match.value(operation).pipe(
		Match.tags({
			CommitMove: (x) => x.side,
			CommitCreate: (x) => x.side,
			CommitCreateFromCommittedChanges: (x) => x.side,
		}),
		Match.orElse(() => null),
	);

export const operationLabel = (operation: Operation): string =>
	Match.value(operation).pipe(
		Match.tagsExhaustive({
			AssignHunk: (operation) => (operation.assignments[0]?.target == null ? "Unassign" : "Assign"),
			CommitAmend: () => "Amend",
			CommitCreate: ({ side }) =>
				Match.value(side).pipe(
					Match.when("above", () => "Create commit above"),
					Match.when("below", () => "Create commit below"),
					Match.exhaustive,
				),
			CommitCreateFromCommittedChanges: ({ side }) =>
				Match.value(side).pipe(
					Match.when("above", () => "Create commit above"),
					Match.when("below", () => "Create commit below"),
					Match.exhaustive,
				),
			CommitMove: ({ side }) =>
				Match.value(side).pipe(
					Match.when("above", () => "Move commit above"),
					Match.when("below", () => "Move commit below"),
					Match.exhaustive,
				),
			CommitMoveChangesBetween: () => "Amend",
			CommitSquash: () => "Squash",
			CommitUncommit: () => "Uncommit",
			CommitUncommitChanges: () => "Uncommit",
			MoveBranch: () => "Stack branch onto here",
			TearOffBranch: () => "Tear off branch",
		}),
	);

export const useRunOperation = () => {
	const toastManager = Toast.useToastManager();
	const assignHunk = useMutation(assignHunkMutationOptions);
	const commitAmend = useMutation(commitAmendMutationOptions);
	const commitCreate = useMutation(commitCreateMutationOptions);
	const commitInsertBlank = useMutation(commitInsertBlankMutationOptions);
	const commitMove = useMutation(commitMoveMutationOptions);
	const commitMoveChangesBetween = useMutation(commitMoveChangesBetweenMutationOptions);
	const commitSquash = useMutation(commitSquashMutationOptions);
	const commitUncommit = useMutation(commitUncommitMutationOptions);
	const commitUncommitChanges = useMutation(commitUncommitChangesMutationOptions);
	const moveBranch = useMutation(moveBranchMutationOptions);
	const tearOffBranch = useMutation(tearOffBranchMutationOptions);

	return (projectId: string, operation: Operation): void => {
		Match.value(operation).pipe(
			Match.tagsExhaustive({
				AssignHunk: (operation) => {
					assignHunk.mutate({
						projectId,
						assignments: operation.assignments,
					});
				},
				CommitAmend: (operation) => {
					commitAmend.mutate(
						{
							projectId,
							commitId: operation.commitId,
							changes: operation.changes,
						},
						{
							onSuccess: (response) => {
								if (response.rejectedChanges.length > 0)
									toastManager.add(
										rejectedChangesToastOptions({
											newCommit: response.newCommit ?? null,
											rejectedChanges: response.rejectedChanges,
										}),
									);
							},
						},
					);
				},
				CommitMoveChangesBetween: (operation) => {
					commitMoveChangesBetween.mutate({
						projectId,
						sourceCommitId: operation.sourceCommitId,
						destinationCommitId: operation.destinationCommitId,
						changes: operation.changes,
					});
				},
				CommitSquash: (operation) => {
					commitSquash.mutate({
						projectId,
						sourceCommitId: operation.sourceCommitId,
						destinationCommitId: operation.destinationCommitId,
					});
				},
				CommitUncommit: (operation) => {
					commitUncommit.mutate({
						projectId,
						commitId: operation.commitId,
						assignTo: operation.assignTo,
					});
				},
				CommitUncommitChanges: (operation) => {
					commitUncommitChanges.mutate({
						projectId,
						commitId: operation.commitId,
						assignTo: operation.assignTo,
						changes: operation.changes,
					});
				},
				CommitCreate: (operation) => {
					commitCreate.mutate(
						{
							projectId,
							relativeTo: operation.relativeTo,
							side: operation.side,
							changes: operation.changes,
							message: operation.message,
						},
						{
							onSuccess: (response) => {
								if (response.rejectedChanges.length > 0)
									toastManager.add(
										rejectedChangesToastOptions({
											newCommit: response.newCommit,
											rejectedChanges: response.rejectedChanges,
										}),
									);
							},
						},
					);
				},
				CommitCreateFromCommittedChanges: (operation) => {
					// Ideally this would be an atomic backend operation.
					void (async () => {
						const insertedCommit = await commitInsertBlank.mutateAsync({
							projectId,
							relativeTo: operation.relativeTo,
							side: operation.side,
						});

						await commitMoveChangesBetween.mutateAsync({
							projectId,
							sourceCommitId:
								insertedCommit.replacedCommits[operation.sourceCommitId] ??
								operation.sourceCommitId,
							destinationCommitId: insertedCommit.newCommit,
							changes: operation.changes,
						});
					})();
				},
				CommitMove: (operation) => {
					commitMove.mutate({
						projectId,
						subjectCommitId: operation.subjectCommitId,
						relativeTo: operation.relativeTo,
						side: operation.side,
					});
				},
				MoveBranch: (operation) => {
					moveBranch.mutate({
						projectId,
						subjectBranch: operation.subjectBranch,
						targetBranch: operation.targetBranch,
					});
				},
				TearOffBranch: (operation) => {
					tearOffBranch.mutate({
						projectId,
						subjectBranch: operation.subjectBranch,
					});
				},
			}),
		);
	};
};
