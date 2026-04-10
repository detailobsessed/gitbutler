import {
	commitDiscardMutationOptions,
	commitInsertBlankMutationOptions,
	unapplyStackMutationOptions,
} from "#ui/api/mutations.ts";
import { useRunOperation } from "#ui/Operation.ts";
import { projectActions } from "#ui/routes/project/$id/state/projectSlice.ts";
import { useAppDispatch } from "#ui/state/hooks.ts";
import type { AppDispatch } from "#ui/state/store.ts";
import type { AbsorptionTarget } from "@gitbutler/but-sdk";
import { Data, Match } from "effect";
import type { RefObject } from "react";
import { useMutation } from "@tanstack/react-query";
import {
	changesSectionItem,
	getParentSection,
	itemIdentityKey,
	type CommitItem,
	type Item as WorkspaceItem,
	type SegmentItem,
	commitItem,
	segmentItem,
} from "./Item.ts";
import { operationModeToOperation } from "./OperationMode.tsx";
import {
	operationSourceIdentityKey,
	operationSourceFromItem,
	type OperationSource,
} from "./OperationSource.ts";
import {
	useResolveOperationSource,
	type ResolvedOperationSource,
} from "./ResolvedOperationSource.ts";
import { getAdjacentItem, getAdjacentSection, type NavigationIndex } from "./WorkspaceModel.ts";
import { type OperationMode } from "./WorkspaceMode.ts";
import { CommitDiscardParams, CommitInsertBlankParams, UnapplyStackParams } from "#electron/ipc.ts";

type PreviewController = {
	moveSelection: (offset: -1 | 1) => void;
};

type Absorb = { item: WorkspaceItem };
type CancelMode = {};
type CloseCommitFiles = { item: CommitItem };
type ClosePreview = {};
type DeleteCommit = { item: CommitItem };
type EnterMoveMode = { source: OperationSource };
type EnterRubMode = { source: OperationSource };
type FocusPreview = {};
type FocusPrimary = {};
type InsertBlankCommitAbove = { item: CommitItem };
type InsertBlankCommitBelow = { item: CommitItem };
type MovePreviewSelection = { offset: -1 | 1 };
type RunOperation = { target: WorkspaceItem | null };
type SelectItem = { item: WorkspaceItem | null };
type SelectNextSection = { item: WorkspaceItem };
type SelectPreviousSection = { item: WorkspaceItem };
type SelectRelativeItem = { item: WorkspaceItem; offset: -1 | 1 };
type SelectUnassignedChanges = {};
type StartRenameBranch = { item: SegmentItem };
type StartRewordCommit = { item: CommitItem };
type SubmitBranchRename = {};
type SubmitCommitMessage = {};
type ToggleCommitFiles = { item: CommitItem };
type ToggleFullscreenPreview = {};
type TogglePreview = {};
type UnapplyStack = { stackId: string };

/** @public */
export type WorkspaceCommand = Data.TaggedEnum<{
	Absorb: Absorb;
	CancelMode: CancelMode;
	CloseCommitFiles: CloseCommitFiles;
	ClosePreview: ClosePreview;
	DeleteCommit: DeleteCommit;
	EnterMoveMode: EnterMoveMode;
	EnterRubMode: EnterRubMode;
	FocusPreview: FocusPreview;
	FocusPrimary: FocusPrimary;
	InsertBlankCommitAbove: InsertBlankCommitAbove;
	InsertBlankCommitBelow: InsertBlankCommitBelow;
	MovePreviewSelection: MovePreviewSelection;
	RunOperation: RunOperation;
	SelectItem: SelectItem;
	SelectNextSection: SelectNextSection;
	SelectPreviousSection: SelectPreviousSection;
	SelectRelativeItem: SelectRelativeItem;
	SelectUnassignedChanges: SelectUnassignedChanges;
	StartRenameBranch: StartRenameBranch;
	StartRewordCommit: StartRewordCommit;
	SubmitBranchRename: SubmitBranchRename;
	SubmitCommitMessage: SubmitCommitMessage;
	ToggleCommitFiles: ToggleCommitFiles;
	ToggleFullscreenPreview: ToggleFullscreenPreview;
	TogglePreview: TogglePreview;
	UnapplyStack: UnapplyStack;
}>;

export const WorkspaceCommand = Data.taggedEnum<WorkspaceCommand>();

export const getWorkspaceCommandLabel = (command: WorkspaceCommand): string =>
	Match.value(command).pipe(
		Match.tagsExhaustive({
			Absorb: () => "Absorb",
			CancelMode: () => "Cancel",
			CloseCommitFiles: () => "Close",
			ClosePreview: () => "Close",
			DeleteCommit: () => "Delete commit",
			EnterMoveMode: () => "Move",
			EnterRubMode: () => "Rub",
			FocusPreview: () => "Focus preview",
			FocusPrimary: () => "Focus primary",
			InsertBlankCommitAbove: () => "Add empty commit above",
			InsertBlankCommitBelow: () => "Add empty commit below",
			MovePreviewSelection: ({ offset }) => (offset < 0 ? "up" : "down"),
			RunOperation: () => "Run",
			SelectItem: () => "Select item",
			SelectNextSection: () => "Next section",
			SelectPreviousSection: () => "Previous section",
			SelectRelativeItem: ({ offset }) => (offset < 0 ? "up" : "down"),
			SelectUnassignedChanges: () => "Unassigned changes",
			StartRenameBranch: () => "Rename",
			StartRewordCommit: () => "Reword",
			SubmitBranchRename: () => "Save",
			SubmitCommitMessage: () => "Save",
			ToggleCommitFiles: () => "Files",
			ToggleFullscreenPreview: () => "Fullscreen preview",
			TogglePreview: () => "Preview",
			UnapplyStack: () => "Unapply stack",
		}),
	);

const workspaceCommandIdentityKey = (command: WorkspaceCommand): string =>
	Match.value(command).pipe(
		Match.tagsExhaustive({
			Absorb: ({ item }) => JSON.stringify(["Absorb", itemIdentityKey(item)]),
			CancelMode: () => JSON.stringify(["CancelMode"]),
			CloseCommitFiles: ({ item }) =>
				JSON.stringify(["CloseCommitFiles", itemIdentityKey(commitItem(item))]),
			ClosePreview: () => JSON.stringify(["ClosePreview"]),
			DeleteCommit: ({ item }) =>
				JSON.stringify(["DeleteCommit", itemIdentityKey(commitItem(item))]),
			EnterMoveMode: ({ source }) =>
				JSON.stringify(["EnterMoveMode", operationSourceIdentityKey(source)]),
			EnterRubMode: ({ source }) =>
				JSON.stringify(["EnterRubMode", operationSourceIdentityKey(source)]),
			FocusPreview: () => JSON.stringify(["FocusPreview"]),
			FocusPrimary: () => JSON.stringify(["FocusPrimary"]),
			InsertBlankCommitAbove: ({ item }) =>
				JSON.stringify(["InsertBlankCommitAbove", itemIdentityKey(commitItem(item))]),
			InsertBlankCommitBelow: ({ item }) =>
				JSON.stringify(["InsertBlankCommitBelow", itemIdentityKey(commitItem(item))]),
			MovePreviewSelection: ({ offset }) => JSON.stringify(["MovePreviewSelection", offset]),
			RunOperation: ({ target }) =>
				JSON.stringify(["RunOperation", target === null ? null : itemIdentityKey(target)]),
			SelectItem: ({ item }) =>
				JSON.stringify(["SelectItem", item === null ? null : itemIdentityKey(item)]),
			SelectNextSection: ({ item }) => JSON.stringify(["SelectNextSection", itemIdentityKey(item)]),
			SelectPreviousSection: ({ item }) =>
				JSON.stringify(["SelectPreviousSection", itemIdentityKey(item)]),
			SelectRelativeItem: ({ item, offset }) =>
				JSON.stringify(["SelectRelativeItem", itemIdentityKey(item), offset]),
			SelectUnassignedChanges: () => JSON.stringify(["SelectUnassignedChanges"]),
			StartRenameBranch: ({ item }) =>
				JSON.stringify(["StartRenameBranch", itemIdentityKey(segmentItem(item))]),
			StartRewordCommit: ({ item }) =>
				JSON.stringify(["StartRewordCommit", itemIdentityKey(commitItem(item))]),
			SubmitBranchRename: () => JSON.stringify(["SubmitBranchRename"]),
			SubmitCommitMessage: () => JSON.stringify(["SubmitCommitMessage"]),
			ToggleCommitFiles: ({ item }) =>
				JSON.stringify(["ToggleCommitFiles", itemIdentityKey(commitItem(item))]),
			ToggleFullscreenPreview: () => JSON.stringify(["ToggleFullscreenPreview"]),
			TogglePreview: () => JSON.stringify(["TogglePreview"]),
			UnapplyStack: ({ stackId }) => JSON.stringify(["UnapplyStack", stackId]),
		}),
	);

export const workspaceCommandEquals = (a: WorkspaceCommand, b: WorkspaceCommand): boolean =>
	workspaceCommandIdentityKey(a) === workspaceCommandIdentityKey(b);

type RunWorkspaceCommandInputs = {
	branchRenameFormRef: RefObject<HTMLFormElement | null>;
	commitMessageFormRef: RefObject<HTMLFormElement | null>;
	navigationIndex: NavigationIndex;
	operationMode: OperationMode | null;
	previewRef: RefObject<PreviewController | null>;
	projectId: string;
	requestAbsorptionPlan: (target: AbsorptionTarget) => void;
};

type ExecuteWorkspaceCommandDependencies = RunWorkspaceCommandInputs & {
	deleteCommit: (input: CommitDiscardParams) => void;
	dispatch: AppDispatch;
	insertBlankCommit: (input: CommitInsertBlankParams) => void;
	resolveOperationSource: (source: OperationSource) => ResolvedOperationSource | null;
	runOperation: ReturnType<typeof useRunOperation>;
	unapplyStack: (input: UnapplyStackParams) => void;
};

const executeWorkspaceCommand = (
	command: WorkspaceCommand,
	dependencies: ExecuteWorkspaceCommandDependencies,
) => {
	const {
		branchRenameFormRef,
		commitMessageFormRef,
		deleteCommit,
		dispatch,
		insertBlankCommit,
		navigationIndex,
		operationMode,
		previewRef,
		projectId,
		requestAbsorptionPlan,
		resolveOperationSource,
		runOperation,
		unapplyStack,
	} = dependencies;

	Match.value(command).pipe(
		Match.tagsExhaustive({
			Absorb: ({ item }) => {
				const resolvedOperationSource = resolveOperationSource(operationSourceFromItem(item));
				if (resolvedOperationSource?._tag !== "TreeChanges") return;
				if (resolvedOperationSource.parent._tag !== "ChangesSection") return;

				requestAbsorptionPlan({
					type: "treeChanges",
					subject: {
						changes: resolvedOperationSource.changes.map(({ change }) => change),
						assigned_stack_id: resolvedOperationSource.parent.stackId,
					},
				});
			},
			CancelMode: () => dispatch(projectActions.exitMode({ projectId })),
			CloseCommitFiles: ({ item }) =>
				dispatch(projectActions.closeCommitFiles({ projectId, item })),
			ClosePreview: () => dispatch(projectActions.closePreview({ projectId })),
			DeleteCommit: ({ item }) =>
				deleteCommit({
					projectId,
					subjectCommitId: item.commitId,
				}),
			EnterMoveMode: ({ source }) => dispatch(projectActions.enterMoveMode({ projectId, source })),
			EnterRubMode: ({ source }) => dispatch(projectActions.enterRubMode({ projectId, source })),
			FocusPreview: () => dispatch(projectActions.focusPreview({ projectId })),
			FocusPrimary: () => dispatch(projectActions.focusPrimary({ projectId })),
			InsertBlankCommitAbove: ({ item }) =>
				insertBlankCommit({
					projectId,
					relativeTo: { type: "commit", subject: item.commitId },
					side: "above",
				}),
			InsertBlankCommitBelow: ({ item }) =>
				insertBlankCommit({
					projectId,
					relativeTo: { type: "commit", subject: item.commitId },
					side: "below",
				}),
			MovePreviewSelection: ({ offset }) => previewRef.current?.moveSelection(offset),
			RunOperation: ({ target }) => {
				dispatch(projectActions.exitMode({ projectId }));
				if (!operationMode || target === null) return;

				const resolvedOperationModeSource = resolveOperationSource(operationMode.source);
				if (!resolvedOperationModeSource) return;

				const operation = operationModeToOperation({
					operationMode,
					resolvedOperationSource: resolvedOperationModeSource,
					target,
				});
				if (!operation) return;

				runOperation(projectId, operation);
			},
			SelectItem: ({ item }) => dispatch(projectActions.selectItem({ projectId, item })),
			SelectNextSection: ({ item }) =>
				dispatch(
					projectActions.selectItem({
						projectId,
						item: getAdjacentSection(navigationIndex, item, 1) ?? null,
					}),
				),
			SelectPreviousSection: ({ item }) =>
				dispatch(
					projectActions.selectItem({
						projectId,
						item: getParentSection(item) ?? getAdjacentSection(navigationIndex, item, -1),
					}),
				),
			SelectRelativeItem: ({ item, offset }) =>
				dispatch(
					projectActions.selectItem({
						projectId,
						item: getAdjacentItem(navigationIndex, item, offset) ?? null,
					}),
				),
			SelectUnassignedChanges: () =>
				dispatch(
					projectActions.selectItem({
						projectId,
						item: changesSectionItem({ stackId: null }),
					}),
				),
			StartRenameBranch: ({ item }) =>
				dispatch(projectActions.startRenameBranch({ projectId, item })),
			StartRewordCommit: ({ item }) =>
				dispatch(projectActions.startRewordCommit({ projectId, item })),
			SubmitBranchRename: () => branchRenameFormRef.current?.requestSubmit(),
			SubmitCommitMessage: () => commitMessageFormRef.current?.requestSubmit(),
			ToggleCommitFiles: ({ item }) =>
				dispatch(projectActions.toggleCommitFiles({ projectId, item })),
			ToggleFullscreenPreview: () =>
				dispatch(projectActions.toggleFullscreenPreview({ projectId })),
			TogglePreview: () => dispatch(projectActions.togglePreview({ projectId })),
			UnapplyStack: ({ stackId }) => unapplyStack({ projectId, stackId }),
		}),
	);
};

export const useRunWorkspaceCommand = ({
	branchRenameFormRef,
	commitMessageFormRef,
	navigationIndex,
	operationMode,
	previewRef,
	projectId,
	requestAbsorptionPlan,
}: RunWorkspaceCommandInputs) => {
	const dispatch = useAppDispatch();
	const commitDiscard = useMutation(commitDiscardMutationOptions);
	const commitInsertBlank = useMutation(commitInsertBlankMutationOptions);
	const resolveOperationSource = useResolveOperationSource(projectId);
	const runOperation = useRunOperation();
	const unapplyStack = useMutation(unapplyStackMutationOptions);

	return (command: WorkspaceCommand) =>
		executeWorkspaceCommand(command, {
			branchRenameFormRef,
			commitMessageFormRef,
			deleteCommit: commitDiscard.mutate,
			dispatch,
			insertBlankCommit: commitInsertBlank.mutate,
			navigationIndex,
			operationMode,
			previewRef,
			projectId,
			requestAbsorptionPlan,
			resolveOperationSource,
			runOperation,
			unapplyStack: unapplyStack.mutate,
		});
};
