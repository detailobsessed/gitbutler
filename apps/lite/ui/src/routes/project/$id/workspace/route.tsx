import { commitRewordMutationOptions, updateBranchNameMutationOptions } from "#ui/api/mutations.ts";
import {
	branchDetailsQueryOptions,
	branchDiffQueryOptions,
	changesInWorktreeQueryOptions,
	commitDetailsWithLineStatsQueryOptions,
	headInfoQueryOptions,
	listProjectsQueryOptions,
	treeChangeDiffsQueryOptions,
} from "#ui/api/queries.ts";
import { classes } from "#ui/classes.ts";
import {
	DependencyIcon,
	ExpandCollapseIcon,
	MenuTriggerIcon,
	PushIcon,
} from "#ui/components/icons.tsx";
import {
	changesSectionFileParent,
	commitFileParent,
	type FileParent,
} from "#ui/domain/FileParent.ts";
import { getBranchNameByCommitId, getCommonBaseCommitId } from "#ui/domain/RefInfo.ts";
import { ProjectPreviewLayout } from "#ui/routes/project/$id/ProjectPreviewLayout.tsx";
import { getFocus, isPreviewPanelVisible } from "#ui/routes/project/$id/state/layout.ts";
import { PositionedTopBarActions } from "#ui/routes/project/$id/TopBarActions.tsx";
import {
	projectActions,
	selectProjectExpandedCommitId,
	selectProjectHighlightedCommitIds,
	selectProjectLayoutState,
	selectProjectSelectedHunk,
	selectProjectSelectedItem,
	selectProjectWorkspaceModeState,
} from "#ui/routes/project/$id/state/projectSlice.ts";
import { normalizeSelectedHunk } from "#ui/routes/project/$id/state/workspace.ts";
import { AbsorptionDialog, useAbsorption } from "#ui/routes/project/$id/workspace/Absorption.tsx";
import { useMonitorDraggedOperationSource } from "#ui/routes/project/$id/workspace/DragAndDrop.tsx";
import { isOperationModeSourceOrTarget } from "#ui/routes/project/$id/workspace/OperationMode.tsx";
import { OperationSourceC } from "#ui/routes/project/$id/workspace/OperationSourceC.tsx";
import { useResolveOperationSource } from "#ui/routes/project/$id/workspace/ResolvedOperationSource.ts";
import {
	CommitTarget,
	OperationTarget,
} from "#ui/routes/project/$id/workspace/OperationTargets.tsx";
import { OperationSourceLabel } from "#ui/routes/project/$id/workspace/OperationSourceLabel.tsx";
import {
	CommitFiles as SharedCommitFiles,
	CommitsList,
	FileButton,
	formatHunkHeader,
	HunkDiff,
	Patch,
	CommitLabel,
	shortCommitId,
	assignedHunks,
	decodeRefName,
	getAssignmentsByPath,
	getRelative,
	hunkKey,
	encodeRefName,
} from "#ui/routes/project/$id/shared.tsx";
import uiStyles from "#ui/ui.module.css";
import { ContextMenu, Menu, mergeProps, Tooltip, useRender } from "@base-ui/react";
import {
	Commit,
	DiffHunk,
	HunkAssignment,
	HunkDependencies,
	HunkHeader,
	Segment,
	Stack,
	TreeChange,
	UnifiedPatch,
} from "@gitbutler/but-sdk";
import { useMutation, useSuspenseQueries, useSuspenseQuery } from "@tanstack/react-query";
import { createRoute } from "@tanstack/react-router";
import { Array, Match, pipe } from "effect";
import { isNonEmptyArray, NonEmptyArray } from "effect/Array";
import {
	ComponentProps,
	FC,
	Fragment,
	ReactNode,
	Ref,
	Suspense,
	useEffect,
	useEffectEvent,
	use,
	useImperativeHandle,
	useOptimistic,
	useRef,
	useState,
	useTransition,
} from "react";
import { Route as projectRoute } from "#ui/routes/project/$id/route.tsx";
import { useAppDispatch, useAppSelector } from "#ui/state/hooks.ts";
import sharedStyles from "../shared.module.css";
import {
	baseCommitItem,
	changeItem,
	changesSectionItem,
	CommitFileItem,
	commitFileItem,
	CommitItem,
	commitItem,
	type Item,
	SegmentItem,
	segmentItem,
} from "./Item.ts";
import {
	buildNavigationIndex,
	filterNavigationIndex,
	getDefaultItem,
	navigationIndexIncludes,
	type NavigationIndex,
	useWorkspaceOutline,
} from "./WorkspaceModel.ts";
import { getScope, useWorkspaceShortcuts } from "./WorkspaceShortcuts.ts";
import { WorkspaceCommandButton } from "./WorkspaceCommandButton.tsx";
import { WorkspaceCommandMenuItem } from "./WorkspaceCommandMenuItem.tsx";
import { WorkspaceCommandRuntimeContext } from "#ui/routes/project/$id/workspace/WorkspaceCommandRuntime.tsx";
import { PositionedShortcutsBar } from "../ShortcutsBar.tsx";
import { formatShortcutKeys } from "#ui/shortcuts.ts";
import { WorkspaceCommandPalette } from "./WorkspaceCommandPalette.tsx";
import styles from "./route.module.css";
import {
	fileOperationSource,
	hunkOperationSource,
	operationSourceFromItem,
} from "./OperationSource.ts";
import { useRunWorkspaceCommand, WorkspaceCommand } from "./WorkspaceCommands.ts";
import {
	getOperationMode,
	normalizeWorkspaceMode,
	type OperationMode,
	type WorkspaceMode,
} from "./WorkspaceMode.ts";
import { assert } from "#ui/routes/project/$id/shared.tsx";

type HunkDependencyDiff = HunkDependencies["diffs"][number];
const fileHunkKey = (path: string, hunk: HunkHeader): string => `${path}:${hunkKey(hunk)}`;

const ItemRow: FC<
	{
		isSelected?: boolean;
	} & ComponentProps<"div">
> = ({ className, isSelected, ...props }) => (
	<div
		{...props}
		className={classes(className, sharedStyles.itemRow, isSelected && sharedStyles.itemRowSelected)}
	/>
);

const DependencyIndicator: FC<
	{
		projectId: string;
		commitIds: NonEmptyArray<string>;
	} & useRender.ComponentProps<"button">
> = ({ projectId, commitIds, render, ...props }) => {
	const dispatch = useAppDispatch();
	const { data: headInfo } = useSuspenseQuery(headInfoQueryOptions(projectId));
	// TODO: expensive
	const branchNameByCommitId = getBranchNameByCommitId(headInfo);
	const branchNames = pipe(
		commitIds,
		Array.flatMapNullable((commitId) => branchNameByCommitId.get(commitId)),
		Array.dedupe,
	);
	const tooltip =
		branchNames.length > 0 ? `Depends on ${branchNames.join(", ")}` : "Unknown dependencies";
	const trigger = useRender({
		render,
		defaultTagName: "button",
		props: mergeProps<"button">(props, {
			onMouseEnter: () => {
				dispatch(projectActions.setHighlightedCommitIds({ projectId, commitIds }));
			},
			onMouseLeave: () => {
				dispatch(projectActions.setHighlightedCommitIds({ projectId, commitIds: null }));
			},
			"aria-label": tooltip,
		}),
	});

	return (
		<Tooltip.Root
			// [ref:tooltip-disable-hoverable-popup]
			disableHoverablePopup
		>
			<Tooltip.Trigger render={trigger} />
			<Tooltip.Portal>
				<Tooltip.Positioner sideOffset={8}>
					<Tooltip.Popup className={classes(uiStyles.popup, uiStyles.tooltip)}>
						{tooltip}
					</Tooltip.Popup>
				</Tooltip.Positioner>
			</Tooltip.Portal>
		</Tooltip.Root>
	);
};

const hunkContainsHunk = (a: HunkHeader, b: HunkHeader): boolean =>
	a.oldStart <= b.oldStart &&
	a.oldStart + a.oldLines - 1 >= b.oldStart + b.oldLines - 1 &&
	a.newStart <= b.newStart &&
	a.newStart + a.newLines - 1 >= b.newStart + b.newLines - 1;

const getHunkDependencyDiffsByPath = (
	hunkDependencyDiffs: Array<HunkDependencyDiff>,
): Map<string, Array<HunkDependencyDiff>> => {
	const byPath = new Map<string, Array<HunkDependencyDiff>>();

	for (const hunkDependencyDiff of hunkDependencyDiffs) {
		const [path] = hunkDependencyDiff;
		const pathDependencyDiffs = byPath.get(path);
		if (pathDependencyDiffs) pathDependencyDiffs.push(hunkDependencyDiff);
		else byPath.set(path, [hunkDependencyDiff]);
	}

	return byPath;
};

const dependencyCommitIdsForHunk = (
	hunk: DiffHunk,
	hunkDependencyDiffs: Array<HunkDependencyDiff>,
): Array<string> => {
	const commitIds = new Set<string>();

	for (const [, dependencyHunk, locks] of hunkDependencyDiffs) {
		if (!hunkContainsHunk(hunk, dependencyHunk)) continue;
		for (const dependency of locks) commitIds.add(dependency.commitId);
	}

	return globalThis.Array.from(commitIds);
};

const dependencyCommitIdsForFile = (
	hunkDependencyDiffs: Array<HunkDependencyDiff>,
): Array<string> => {
	const commitIds = new Set<string>();

	for (const [, , locks] of hunkDependencyDiffs)
		for (const dependency of locks) commitIds.add(dependency.commitId);

	return globalThis.Array.from(commitIds);
};

const Hunk: FC<{
	patch: Patch;
	operationMode: OperationMode | null;
	projectId: string;
	fileParent?: FileParent;
	change: TreeChange;
	hunk: DiffHunk;
	editable: boolean;
	headerStart?: ReactNode;
	isSelected?: boolean;
	isFocused: boolean;
}> = ({
	patch,
	operationMode,
	projectId,
	fileParent,
	change,
	hunk,
	editable,
	headerStart,
	isSelected,
	isFocused,
}) => {
	const dispatch = useAppDispatch();
	const headerRow = (
		<div className={sharedStyles.hunkHeaderRow}>
			{headerStart}
			<div className={sharedStyles.hunkHeader}>{formatHunkHeader(hunk)}</div>
		</div>
	);

	return (
		// oxlint-disable-next-line jsx_a11y/click-events-have-key-events, jsx_a11y/no-static-element-interactions -- TODO
		<div
			onClick={() =>
				dispatch(
					projectActions.selectHunk({
						projectId,
						hunk: fileHunkKey(change.path, hunk),
					}),
				)
			}
			className={classes(
				sharedStyles.previewHunk,
				isSelected && isFocused && sharedStyles.previewHunkSelected,
			)}
		>
			{fileParent && editable
				? (() => {
						const source = hunkOperationSource({
							parent: fileParent,
							path: change.path,
							hunkHeader: hunk,
						});
						return (
							<OperationSourceC
								operationMode={operationMode}
								projectId={projectId}
								source={source}
								canDrag={() => !patch.subject.isResultOfBinaryToTextConversion}
							>
								{headerRow}
							</OperationSourceC>
						);
					})()
				: headerRow}
			<HunkDiff change={change} diff={hunk.diff} />
		</div>
	);
};

const FileDiff: FC<{
	operationMode: OperationMode | null;
	projectId: string;
	change: TreeChange;
	assignments?: Array<HunkAssignment>;
	fileParent?: FileParent;
	editable: boolean;
	hunkDependencyDiffs?: Array<HunkDependencyDiff>;
	diff: UnifiedPatch | null;
	selectedHunk: string | undefined;
	isFocused: boolean;
}> = ({
	operationMode,
	projectId,
	change,
	assignments,
	fileParent,
	editable,
	hunkDependencyDiffs,
	diff,
	selectedHunk,
	isFocused,
}) =>
	Match.value(diff).pipe(
		Match.when(null, () => <div>No diff available for this file.</div>),
		Match.when({ type: "Binary" }, () => <div>Binary file (diff not available).</div>),
		Match.when({ type: "TooLarge" }, ({ subject }) => (
			<div>Diff too large ({subject.sizeInBytes} bytes).</div>
		)),
		Match.when({ type: "Patch" }, (patch) => {
			const visibleHunks = assignments
				? assignedHunks(patch.subject.hunks, assignments)
				: patch.subject.hunks;
			if (visibleHunks.length === 0) return <div>No hunks.</div>;

			return (
				<ul>
					{visibleHunks.map((hunk) => {
						const dependencyCommitIds = hunkDependencyDiffs
							? dependencyCommitIdsForHunk(hunk, hunkDependencyDiffs)
							: [];

						return (
							<li key={hunkKey(hunk)}>
								<Hunk
									patch={patch}
									operationMode={operationMode}
									projectId={projectId}
									fileParent={fileParent}
									change={change}
									hunk={hunk}
									editable={editable}
									isSelected={selectedHunk === fileHunkKey(change.path, hunk)}
									isFocused={isFocused}
									headerStart={
										fileParent?._tag === "ChangesSection" &&
										isNonEmptyArray(dependencyCommitIds) ? (
											<DependencyIndicator projectId={projectId} commitIds={dependencyCommitIds}>
												<DependencyIcon />
											</DependencyIndicator>
										) : undefined
									}
								/>
							</li>
						);
					})}
				</ul>
			);
		}),
		Match.exhaustive,
	);

const hunkKeysFromChangeWithDiff = (
	[change, diff]: [TreeChange, UnifiedPatch | null],
	assignments?: Array<HunkAssignment>,
): Array<string> =>
	diff?.type === "Patch"
		? (assignments ? assignedHunks(diff.subject.hunks, assignments) : diff.subject.hunks).map(
				(hunk) => fileHunkKey(change.path, hunk),
			)
		: [];

type PreviewImperativeHandle = {
	moveSelection: (offset: -1 | 1) => void;
};

const usePreviewDiffState = ({
	projectId,
	changes,
	ref,
	getAssignments,
}: {
	projectId: string;
	changes: Array<TreeChange>;
	ref?: Ref<PreviewImperativeHandle>;
	getAssignments?: (path: string) => Array<HunkAssignment> | undefined;
}) => {
	const dispatch = useAppDispatch();
	const selectedHunk = useAppSelector((state) => selectProjectSelectedHunk(state, projectId));
	const treeChangeDiffs = useSuspenseQueries({
		queries: changes.map((change) => treeChangeDiffsQueryOptions({ projectId, change })),
	}).map((result) => result.data);
	const changesWithDiffs = pipe(changes, Array.zip(treeChangeDiffs));
	const hunkKeys = changesWithDiffs.flatMap(([change, diff]) =>
		hunkKeysFromChangeWithDiff([change, diff], getAssignments?.(change.path)),
	);
	const normalizedSelectedHunk = normalizeSelectedHunk({ hunkKeys, selectedHunk });

	useImperativeHandle(
		ref,
		(): PreviewImperativeHandle => ({
			moveSelection: (offset) => {
				if (hunkKeys.length === 0) return;

				if (normalizedSelectedHunk === undefined) return;

				// We assume a valid key was provided.
				const currentIndex = hunkKeys.indexOf(normalizedSelectedHunk);

				dispatch(
					projectActions.selectHunk({
						projectId,
						hunk: getRelative(hunkKeys, currentIndex, offset),
					}),
				);
			},
		}),
		[dispatch, normalizedSelectedHunk, hunkKeys, projectId],
	);

	return { changesWithDiffs, normalizedSelectedHunk };
};

const ChangesPreview: FC<{
	operationMode: OperationMode | null;
	projectId: string;
	stackId: string | null;
	selectedPath?: string;
	isFocused: boolean;
	ref?: Ref<PreviewImperativeHandle>;
}> = ({ operationMode, projectId, stackId, selectedPath, isFocused, ref }) => {
	const { data: worktreeChanges } = useSuspenseQuery(changesInWorktreeQueryOptions(projectId));
	const assignmentsByPath = getAssignmentsByPath(worktreeChanges.assignments, stackId);
	const hunkDependencyDiffsByPath = getHunkDependencyDiffsByPath(
		worktreeChanges.dependencies?.diffs ?? [],
	);
	const changes = worktreeChanges.changes.filter((change) => assignmentsByPath.has(change.path));
	const selectedChange =
		selectedPath !== undefined
			? changes.find((candidate) => candidate.path === selectedPath)
			: undefined;
	const visibleChanges = selectedChange ? [selectedChange] : changes;
	const { changesWithDiffs, normalizedSelectedHunk } = usePreviewDiffState({
		projectId,
		changes: visibleChanges,
		ref,
		getAssignments: (path) => assignmentsByPath.get(path),
	});

	return (
		<div>
			{changesWithDiffs.length === 0 ? (
				<div>No file changes.</div>
			) : (
				<ul>
					{changesWithDiffs.map(([change, diff]) => {
						const parent = changesSectionFileParent({ stackId });
						const source = fileOperationSource({ parent, path: change.path });
						return (
							<li key={change.path}>
								<OperationSourceC
									operationMode={operationMode}
									projectId={projectId}
									source={source}
								>
									<h4>{change.path}</h4>
								</OperationSourceC>
								<FileDiff
									operationMode={operationMode}
									projectId={projectId}
									change={change}
									fileParent={parent}
									editable
									assignments={assignmentsByPath.get(change.path)}
									hunkDependencyDiffs={hunkDependencyDiffsByPath.get(change.path)}
									diff={diff}
									selectedHunk={normalizedSelectedHunk}
									isFocused={isFocused}
								/>
							</li>
						);
					})}
				</ul>
			)}
		</div>
	);
};

const CommitPreview: FC<{
	operationMode: OperationMode | null;
	projectId: string;
	commitId: string;
	selectedPath?: string | null;
	editable: boolean;
	isFocused: boolean;
	ref?: Ref<PreviewImperativeHandle>;
}> = ({ operationMode, projectId, commitId, selectedPath, editable, isFocused, ref }) => {
	const { data: commitDetails } = useSuspenseQuery(
		commitDetailsWithLineStatsQueryOptions({ projectId, commitId }),
	);
	const selectedChange =
		selectedPath !== undefined
			? commitDetails.changes.find((candidate) => candidate.path === selectedPath)
			: undefined;
	const visibleChanges =
		selectedPath === undefined ? commitDetails.changes : selectedChange ? [selectedChange] : [];
	const { changesWithDiffs, normalizedSelectedHunk } = usePreviewDiffState({
		projectId,
		changes: visibleChanges,
		ref,
	});

	return (
		<div>
			{selectedPath === undefined && (
				<>
					<h3>
						<CommitLabel commit={commitDetails.commit} />
					</h3>
					{commitDetails.commit.message.includes("\n") && (
						<p className={sharedStyles.commitMessageBody}>
							{commitDetails.commit.message
								.slice(commitDetails.commit.message.indexOf("\n") + 1)
								.trim()}
						</p>
					)}
				</>
			)}
			{changesWithDiffs.length === 0 ? (
				<div>No file changes.</div>
			) : (
				<ul>
					{changesWithDiffs.map(([change, diff]) => {
						const parent = commitFileParent({ commitId });
						const source = fileOperationSource({ parent, path: change.path });
						return (
							<li key={change.path}>
								{editable ? (
									<OperationSourceC
										operationMode={operationMode}
										projectId={projectId}
										source={source}
									>
										<h4>{change.path}</h4>
									</OperationSourceC>
								) : (
									<h4>{change.path}</h4>
								)}
								<FileDiff
									operationMode={operationMode}
									projectId={projectId}
									change={change}
									fileParent={parent}
									editable={editable}
									diff={diff}
									selectedHunk={normalizedSelectedHunk}
									isFocused={isFocused}
								/>
							</li>
						);
					})}
				</ul>
			)}
		</div>
	);
};

const BranchPreview: FC<{
	operationMode: OperationMode | null;
	projectId: string;
	branchRef: Array<number>;
	isFocused: boolean;
	ref?: Ref<PreviewImperativeHandle>;
}> = ({ operationMode, projectId, branchRef, isFocused, ref }) => {
	const branch = decodeRefName(branchRef);
	const [{ data: branchDetails }, { data: branchDiff }] = useSuspenseQueries({
		queries: [
			branchDetailsQueryOptions({
				projectId,
				// https://linear.app/gitbutler/issue/GB-1226/unify-branch-identifiers
				branchName: branch.replace(/^refs\/heads\//, ""),
				remote: null,
			}),
			branchDiffQueryOptions({ projectId, branch }),
		],
	});
	const { changesWithDiffs, normalizedSelectedHunk } = usePreviewDiffState({
		projectId,
		changes: branchDiff.changes,
		ref,
	});

	return (
		<div>
			<h3>{branchDetails.name}</h3>
			{branchDetails.prNumber != null && <p>PR #{branchDetails.prNumber}</p>}
			{changesWithDiffs.length === 0 ? (
				<div>No file changes.</div>
			) : (
				<ul>
					{changesWithDiffs.map(([change, diff]) => (
						<li key={change.path}>
							<h4>{change.path}</h4>
							<FileDiff
								operationMode={operationMode}
								projectId={projectId}
								change={change}
								editable={false}
								diff={diff}
								selectedHunk={normalizedSelectedHunk}
								isFocused={isFocused}
							/>
						</li>
					))}
				</ul>
			)}
		</div>
	);
};

const Preview: FC<{
	operationMode: OperationMode | null;
	projectId: string;
	selectedItem: Item;
	isFocused: boolean;
	ref?: Ref<PreviewImperativeHandle>;
}> = ({ operationMode, projectId, selectedItem, isFocused, ref }) =>
	Match.value(selectedItem).pipe(
		Match.tagsExhaustive({
			Segment: ({ branchRef }) => {
				if (branchRef == null)
					return (
						<div>
							TODO: the API doesn't provide a way to show details/diffs for segments that don't have
							branch names.
						</div>
					);

				return (
					<BranchPreview
						operationMode={operationMode}
						projectId={projectId}
						branchRef={branchRef}
						isFocused={isFocused}
						ref={ref}
					/>
				);
			},
			ChangesSection: ({ stackId }) => (
				<ChangesPreview
					operationMode={operationMode}
					projectId={projectId}
					stackId={stackId}
					isFocused={isFocused}
					ref={ref}
				/>
			),
			Change: ({ stackId, path }) => (
				<ChangesPreview
					operationMode={operationMode}
					projectId={projectId}
					stackId={stackId}
					selectedPath={path}
					isFocused={isFocused}
					ref={ref}
				/>
			),
			Commit: (selectedItem) => (
				<CommitPreview
					operationMode={operationMode}
					projectId={projectId}
					commitId={selectedItem.commitId}
					editable
					isFocused={isFocused}
					ref={ref}
				/>
			),
			CommitFile: ({ commitId, path }) => (
				<CommitPreview
					operationMode={operationMode}
					projectId={projectId}
					commitId={commitId}
					selectedPath={path}
					editable
					isFocused={isFocused}
					ref={ref}
				/>
			),
			BaseCommit: () => null,
		}),
	);

const StackMenuPopup: FC<{
	stackId: string;
}> = ({ stackId }) => (
	<Menu.Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
		<Menu.Item className={uiStyles.menuItem} disabled>
			Move to leftmost
		</Menu.Item>
		<Menu.Item className={uiStyles.menuItem} disabled>
			Move to rightmost
		</Menu.Item>
		<Menu.Separator />
		<Menu.Item
			className={uiStyles.menuItem}
			render={<WorkspaceCommandMenuItem command={WorkspaceCommand.UnapplyStack({ stackId })} />}
		/>
	</Menu.Popup>
);

const EditorHelp: FC = () => {
	const { scope } = assert(use(WorkspaceCommandRuntimeContext));
	const bindings = scope ? scope.bindings : [];

	if (bindings.length === 0) return null;

	return (
		<div className={styles.editorHelp}>
			{bindings.map((binding, index) => (
				<Fragment key={binding.id}>
					{index > 0 && " • "}
					<span className={styles.editorShortcut}>{formatShortcutKeys(binding.keys)}</span> to{" "}
					{binding.description}
				</Fragment>
			))}
		</div>
	);
};

const InlineCommitMessageEditor: FC<{
	message: string;
	onSubmit: (value: string) => void;
	onCancel: () => void;
	formRef?: Ref<HTMLFormElement>;
}> = ({ message, onSubmit, onCancel, formRef }) => (
	<form
		ref={formRef}
		className={styles.editorForm}
		onSubmit={(event) => {
			event.preventDefault();
			const formData = new FormData(event.currentTarget);
			onCancel();
			onSubmit(formData.get("message") as string);
		}}
	>
		<textarea
			ref={(el) => {
				if (!el) return;
				el.focus();
				const cursorPosition = el.value.length;
				el.setSelectionRange(cursorPosition, cursorPosition);
			}}
			aria-label="Commit message"
			name="message"
			defaultValue={message.trim()}
			className={classes(styles.editorInput, styles.editCommitMessageInput)}
		/>
		<EditorHelp />
	</form>
);

const CommitMenuPopup: FC<{
	item: CommitItem;
	canReword: boolean;
	parts: typeof Menu | typeof ContextMenu;
}> = ({ item, canReword, parts }) => {
	const { Item, Popup, SubmenuRoot, SubmenuTrigger, Positioner } = parts;

	return (
		<Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
			<Item
				className={uiStyles.menuItem}
				disabled={!canReword}
				render={<WorkspaceCommandMenuItem command={WorkspaceCommand.StartRewordCommit({ item })} />}
			/>
			<SubmenuRoot>
				<SubmenuTrigger className={uiStyles.menuItem}>Add empty commit</SubmenuTrigger>
				<Positioner>
					<Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
						<Item
							className={uiStyles.menuItem}
							render={
								<WorkspaceCommandMenuItem
									command={WorkspaceCommand.InsertBlankCommitAbove({ item })}
								/>
							}
						/>
						<Item
							className={uiStyles.menuItem}
							render={
								<WorkspaceCommandMenuItem
									command={WorkspaceCommand.InsertBlankCommitBelow({ item })}
								/>
							}
						/>
					</Popup>
				</Positioner>
			</SubmenuRoot>
			<Item
				className={uiStyles.menuItem}
				render={<WorkspaceCommandMenuItem command={WorkspaceCommand.DeleteCommit({ item })} />}
			/>
		</Popup>
	);
};

const CommitRow: FC<
	{
		branchRef: Array<number> | null;
		commit: Commit;
		commitMessageFormRef: Ref<HTMLFormElement>;
		workspaceMode: WorkspaceMode;
		isExpanded: boolean;
		isHighlighted: boolean;
		selected: CommitItem | null;
		projectId: string;
		segmentIndex: number;
		stackId: string;
		navigationIndex: NavigationIndex;
	} & ComponentProps<"div">
> = ({
	branchRef,
	commit,
	commitMessageFormRef,
	workspaceMode,
	isExpanded,
	isHighlighted,
	selected,
	projectId,
	segmentIndex,
	stackId,
	navigationIndex,
	...restProps
}) => {
	const { runCommand } = assert(use(WorkspaceCommandRuntimeContext));
	const commitItemV: CommitItem = {
		stackId,
		segmentIndex,
		branchRef,
		commitId: commit.id,
	};
	const item = commitItem(commitItemV);
	const isRewording =
		selected !== null &&
		workspaceMode._tag === "RewordCommit" &&
		workspaceMode.commitId === commit.id;
	const [optimisticMessage, setOptimisticMessage] = useOptimistic(
		commit.message,
		(_currentMessage, nextMessage: string) => nextMessage,
	);
	const [isCommitMessagePending, startCommitMessageTransition] = useTransition();

	const commitWithOptimisticMessage: Commit = {
		...commit,
		message: optimisticMessage,
	};

	const commitReword = useMutation(commitRewordMutationOptions);

	const endEditing = () => {
		runCommand(WorkspaceCommand.CancelMode());
		runCommand(WorkspaceCommand.SelectItem({ item }));
	};

	const saveNewMessage = (newMessage: string) => {
		const initialMessage = commit.message.trim();
		const trimmed = newMessage.trim();
		if (trimmed === initialMessage) return;
		startCommitMessageTransition(async () => {
			setOptimisticMessage(trimmed);
			try {
				await commitReword.mutateAsync({
					projectId,
					commitId: commit.id,
					message: trimmed,
				});
			} catch {
				// Use the global mutation error handler (shows toast) instead of React
				// error boundaries.
				return;
			}
		});
	};

	return (
		<ItemRow
			{...restProps}
			inert={!navigationIndexIncludes(navigationIndex, item)}
			isSelected={selected !== null}
			className={classes(restProps.className, isHighlighted && sharedStyles.itemRowHighlighted)}
		>
			{isRewording ? (
				<InlineCommitMessageEditor
					formRef={commitMessageFormRef}
					message={optimisticMessage}
					onSubmit={saveNewMessage}
					onCancel={endEditing}
				/>
			) : (
				<>
					<ContextMenu.Root disabled={workspaceMode._tag !== "Default"}>
						<ContextMenu.Trigger
							render={
								<button
									type="button"
									className={classes(
										sharedStyles.commitButton,
										isCommitMessagePending && sharedStyles.commitButtonPending,
									)}
									onClick={() => runCommand(WorkspaceCommand.SelectItem({ item }))}
								>
									<CommitLabel commit={commitWithOptimisticMessage} />
								</button>
							}
						/>
						<ContextMenu.Portal>
							<ContextMenu.Positioner>
								<CommitMenuPopup
									item={commitItemV}
									canReword={!isCommitMessagePending}
									parts={ContextMenu}
								/>
							</ContextMenu.Positioner>
						</ContextMenu.Portal>
					</ContextMenu.Root>
					{workspaceMode._tag === "Default" && (
						<>
							<WorkspaceCommandButton
								command={WorkspaceCommand.ToggleCommitFiles({ item: commitItemV })}
								className={sharedStyles.itemRowAction}
								type="button"
								aria-expanded={isExpanded}
							>
								<ExpandCollapseIcon isExpanded={isExpanded} />
							</WorkspaceCommandButton>
							<Menu.Root>
								<Menu.Trigger className={sharedStyles.itemRowAction} aria-label="Commit menu">
									<MenuTriggerIcon />
								</Menu.Trigger>
								<Menu.Portal>
									<Menu.Positioner align="end">
										<CommitMenuPopup
											item={commitItemV}
											canReword={!isCommitMessagePending}
											parts={Menu}
										/>
									</Menu.Positioner>
								</Menu.Portal>
							</Menu.Root>
						</>
					)}
				</>
			)}
		</ItemRow>
	);
};

const CommitFileRow: FC<{
	change: TreeChange;
	operationMode: OperationMode | null;
	parentCommitItem: CommitItem;
	isSelected: boolean;
	navigationIndex: NavigationIndex;
	projectId: string;
}> = ({ change, operationMode, parentCommitItem, isSelected, navigationIndex, projectId }) => {
	const dispatch = useAppDispatch();
	const item = commitFileItem({ ...parentCommitItem, path: change.path });

	return (
		<OperationSourceC
			operationMode={operationMode}
			projectId={projectId}
			source={operationSourceFromItem(item)}
			render={
				<ItemRow
					inert={!navigationIndexIncludes(navigationIndex, item)}
					isSelected={isSelected}
					className={sharedStyles.fileRow}
				/>
			}
		>
			<FileButton
				change={change}
				onClick={() => {
					dispatch(projectActions.selectItem({ projectId, item }));
				}}
			/>
		</OperationSourceC>
	);
};

const CommitC: FC<{
	branchRef: Array<number> | null;
	commit: Commit;
	commitMessageFormRef: Ref<HTMLFormElement>;
	operationMode: OperationMode | null;
	workspaceMode: WorkspaceMode;
	selected: CommitItem | null;
	selectedFile: CommitFileItem | null;
	projectId: string;
	segmentIndex: number;
	stackId: string;
	navigationIndex: NavigationIndex;
}> = ({
	branchRef,
	commit,
	commitMessageFormRef,
	operationMode,
	workspaceMode,
	selected,
	selectedFile,
	projectId,
	segmentIndex,
	stackId,
	navigationIndex,
}) => {
	const isExpanded = useAppSelector(
		(state) => selectProjectExpandedCommitId(state, projectId) === commit.id,
	);
	const isHighlighted = useAppSelector((state) =>
		selectProjectHighlightedCommitIds(state, projectId).includes(commit.id),
	);
	const commitItemV: CommitItem = { stackId, segmentIndex, branchRef, commitId: commit.id };
	const item = commitItem(commitItemV);
	return (
		<OperationSourceC
			operationMode={operationMode}
			projectId={projectId}
			source={operationSourceFromItem(item)}
			canDrag={() => !selected || workspaceMode._tag !== "RewordCommit"}
			render={
				<CommitTarget
					commitId={commit.id}
					item={item}
					projectId={projectId}
					operationMode={operationMode}
					selectedItem={selected ? item : null}
				/>
			}
		>
			<CommitRow
				branchRef={branchRef}
				commit={commit}
				commitMessageFormRef={commitMessageFormRef}
				workspaceMode={workspaceMode}
				isExpanded={isExpanded}
				isHighlighted={isHighlighted}
				selected={selected}
				projectId={projectId}
				segmentIndex={segmentIndex}
				stackId={stackId}
				navigationIndex={navigationIndex}
			/>
			{isExpanded && (
				<Suspense fallback={<div className={sharedStyles.itemRowEmpty}>Loading commit files…</div>}>
					<SharedCommitFiles
						projectId={projectId}
						commitId={commit.id}
						renderFile={(change) => (
							<CommitFileRow
								change={change}
								operationMode={operationMode}
								parentCommitItem={commitItemV}
								isSelected={selectedFile?.path === change.path}
								navigationIndex={navigationIndex}
								projectId={projectId}
							/>
						)}
					/>
				</Suspense>
			)}
		</OperationSourceC>
	);
};

const ChangeRowMenuPopup: FC<{
	item: Item;
	parts: typeof Menu | typeof ContextMenu;
}> = ({ item, parts }) => {
	const { Item, Popup } = parts;

	return (
		<Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
			<Item
				className={uiStyles.menuItem}
				render={<WorkspaceCommandMenuItem command={WorkspaceCommand.Absorb({ item })} />}
			/>
		</Popup>
	);
};

const ChangeRow: FC<{
	change: TreeChange;
	dependencyCommitIds: Array<string>;
	isSelected: boolean;
	navigationIndex: NavigationIndex;
	operationMode: OperationMode | null;
	projectId: string;
	stackId: string | null;
}> = ({
	change,
	dependencyCommitIds,
	isSelected,
	navigationIndex,
	operationMode,
	projectId,
	stackId,
}) => {
	const dispatch = useAppDispatch();
	const item = changeItem({ stackId, path: change.path });
	return (
		<OperationSourceC
			operationMode={operationMode}
			projectId={projectId}
			source={operationSourceFromItem(item)}
			render={
				<ItemRow inert={!navigationIndexIncludes(navigationIndex, item)} isSelected={isSelected} />
			}
		>
			<ContextMenu.Root>
				<ContextMenu.Trigger
					render={
						<FileButton
							change={change}
							onClick={() => {
								dispatch(projectActions.selectItem({ projectId, item }));
							}}
						/>
					}
				/>
				<ContextMenu.Portal>
					<ContextMenu.Positioner>
						<ChangeRowMenuPopup item={item} parts={ContextMenu} />
					</ContextMenu.Positioner>
				</ContextMenu.Portal>
			</ContextMenu.Root>
			{isNonEmptyArray(dependencyCommitIds) && (
				<DependencyIndicator
					projectId={projectId}
					commitIds={dependencyCommitIds}
					className={sharedStyles.itemRowAction}
				>
					<DependencyIcon />
				</DependencyIndicator>
			)}
			<Menu.Root>
				<Menu.Trigger className={sharedStyles.itemRowAction} aria-label={`${change.path} menu`}>
					<MenuTriggerIcon />
				</Menu.Trigger>
				<Menu.Portal>
					<Menu.Positioner align="end">
						<ChangeRowMenuPopup item={item} parts={Menu} />
					</Menu.Positioner>
				</Menu.Portal>
			</Menu.Root>
		</OperationSourceC>
	);
};

const ChangesSectionRowMenuPopup: FC<{
	canAbsorb: boolean;
	item: Item;
	parts: typeof Menu | typeof ContextMenu;
}> = ({ canAbsorb, item, parts }) => {
	const { Item, Popup } = parts;

	return (
		<Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
			<Item
				className={uiStyles.menuItem}
				disabled={!canAbsorb}
				render={<WorkspaceCommandMenuItem command={WorkspaceCommand.Absorb({ item })} />}
			/>
		</Popup>
	);
};

const ChangesSectionRow: FC<{
	changes: Array<TreeChange>;
	isSelected: boolean;
	navigationIndex: NavigationIndex;
	label: string;
	projectId: string;
	stackId: string | null;
}> = ({ changes, isSelected, navigationIndex, label, projectId, stackId }) => {
	const dispatch = useAppDispatch();
	const item = changesSectionItem({ stackId });

	return (
		<ItemRow inert={!navigationIndexIncludes(navigationIndex, item)} isSelected={isSelected}>
			<ContextMenu.Root>
				<ContextMenu.Trigger
					render={
						<button
							type="button"
							className={styles.segmentButton}
							onClick={() => {
								dispatch(
									projectActions.selectItem({
										projectId,
										item,
									}),
								);
							}}
						>
							{label}
						</button>
					}
				/>
				<ContextMenu.Portal>
					<ContextMenu.Positioner>
						<ChangesSectionRowMenuPopup
							canAbsorb={changes.length !== 0}
							item={item}
							parts={ContextMenu}
						/>
					</ContextMenu.Positioner>
				</ContextMenu.Portal>
			</ContextMenu.Root>
			<Menu.Root>
				<Menu.Trigger className={sharedStyles.itemRowAction} aria-label={`${label} menu`}>
					<MenuTriggerIcon />
				</Menu.Trigger>
				<Menu.Portal>
					<Menu.Positioner align="end">
						<ChangesSectionRowMenuPopup canAbsorb={changes.length !== 0} item={item} parts={Menu} />
					</Menu.Positioner>
				</Menu.Portal>
			</Menu.Root>
		</ItemRow>
	);
};

const BaseCommitRow: FC<
	{
		commitId?: string;
		isSelected: boolean;
		navigationIndex: NavigationIndex;
	} & ComponentProps<"div">
> = ({ commitId, isSelected, navigationIndex, ...props }) => {
	const dispatch = useAppDispatch();
	const projectId = Route.useParams().id;

	return (
		<ItemRow
			{...props}
			inert={!navigationIndexIncludes(navigationIndex, baseCommitItem)}
			isSelected={isSelected}
		>
			<button
				type="button"
				className={styles.commonBaseCommit}
				onClick={() => {
					dispatch(projectActions.selectItem({ projectId, item: baseCommitItem }));
				}}
			>
				{commitId !== undefined
					? `${shortCommitId(commitId)} (common base commit)`
					: "(base commit)"}
			</button>
		</ItemRow>
	);
};

const Changes: FC<{
	label: string;
	operationMode: OperationMode | null;
	projectId: string;
	stackId: string | null;
	isSelected: boolean;
	selectedPath: string | null;
	className?: string;
	navigationIndex: NavigationIndex;
}> = ({
	label,
	operationMode,
	projectId,
	stackId,
	isSelected,
	selectedPath,
	className,
	navigationIndex,
}) => {
	const { data: worktreeChanges } = useSuspenseQuery(changesInWorktreeQueryOptions(projectId));

	const assignmentsByPath = getAssignmentsByPath(worktreeChanges.assignments, stackId);
	const hunkDependencyDiffsByPath = getHunkDependencyDiffsByPath(
		worktreeChanges.dependencies?.diffs ?? [],
	);

	const changes = worktreeChanges.changes.filter((change) => assignmentsByPath.has(change.path));
	const isSectionSelected = isSelected || selectedPath !== null;

	const item = changesSectionItem({ stackId });

	const { runCommand } = assert(use(WorkspaceCommandRuntimeContext));

	const commit = () =>
		runCommand(
			WorkspaceCommand.EnterMoveMode({
				source: operationSourceFromItem(changesSectionItem({ stackId })),
			}),
		);

	return (
		<div className={styles.changesContainer}>
			<OperationSourceC
				operationMode={operationMode}
				projectId={projectId}
				source={operationSourceFromItem(item)}
				className={classes(
					className,
					sharedStyles.section,
					isSectionSelected && sharedStyles.sectionSelected,
				)}
				render={
					<OperationTarget
						item={item}
						projectId={projectId}
						operationMode={operationMode}
						selectedItem={isSelected ? item : null}
					/>
				}
			>
				<ChangesSectionRow
					changes={changes}
					isSelected={isSelected}
					navigationIndex={navigationIndex}
					label={label}
					projectId={projectId}
					stackId={stackId}
				/>
				{changes.length === 0 ? (
					<div className={sharedStyles.itemRowEmpty}>No changes.</div>
				) : (
					<ul>
						{changes.map((change) => {
							const hunkDependencyDiffs = hunkDependencyDiffsByPath.get(change.path);
							const dependencyCommitIds = hunkDependencyDiffs
								? dependencyCommitIdsForFile(hunkDependencyDiffs)
								: [];

							return (
								<li key={change.path}>
									<ChangeRow
										change={change}
										dependencyCommitIds={dependencyCommitIds}
										isSelected={selectedPath === change.path}
										navigationIndex={navigationIndex}
										operationMode={operationMode}
										projectId={projectId}
										stackId={stackId}
									/>
								</li>
							);
						})}
					</ul>
				)}
			</OperationSourceC>
			<button type="button" className={uiStyles.button} onClick={commit}>
				Commit
			</button>
		</div>
	);
};

const SegmentMenuPopup: FC<{
	item: SegmentItem;
	canRename: boolean;
	parts: typeof Menu | typeof ContextMenu;
}> = ({ item, canRename, parts }) => {
	const { Item, Popup } = parts;

	return (
		<Popup className={classes(uiStyles.popup, uiStyles.menuPopup)}>
			<Item
				className={uiStyles.menuItem}
				disabled={!canRename}
				render={<WorkspaceCommandMenuItem command={WorkspaceCommand.StartRenameBranch({ item })} />}
			/>
		</Popup>
	);
};

const InlineBranchNameEditor: FC<{
	branchName: string;
	onSubmit: (value: string) => void;
	onExit: () => void;
	formRef?: Ref<HTMLFormElement>;
}> = ({ branchName, onSubmit, onExit, formRef }) => (
	<form
		ref={formRef}
		className={styles.editorForm}
		onSubmit={(event) => {
			event.preventDefault();
			const formData = new FormData(event.currentTarget);
			onExit();
			onSubmit(formData.get("branchName") as string);
		}}
	>
		<input
			aria-label="Branch name"
			ref={(el) => {
				if (!el) return;
				el.focus();
				el.select();
			}}
			name="branchName"
			defaultValue={branchName}
			className={classes(styles.editorInput, styles.renameBranchInput)}
		/>
		<EditorHelp />
	</form>
);

const SegmentRow: FC<
	{
		branchRenameFormRef: Ref<HTMLFormElement>;
		operationMode: OperationMode | null;
		selected: SegmentItem | null;
		workspaceMode: WorkspaceMode;
		projectId: string;
		segment: Segment;
		stackId: string;
		segmentIndex: number;
		navigationIndex: NavigationIndex;
	} & ComponentProps<"div">
> = ({
	branchRenameFormRef,
	operationMode,
	selected,
	workspaceMode,
	projectId,
	segment,
	stackId,
	segmentIndex,
	navigationIndex,
	...restProps
}) => {
	const { runCommand } = assert(use(WorkspaceCommandRuntimeContext));
	const branchName = segment.refName?.displayName ?? null;
	const branchRef = segment.refName?.fullNameBytes ?? null;
	const segmentItemV: SegmentItem = {
		stackId,
		segmentIndex,
		branchRef,
	};
	const item = segmentItem(segmentItemV);
	const isRenaming =
		selected !== null &&
		workspaceMode._tag === "RenameBranch" &&
		workspaceMode.stackId === stackId &&
		workspaceMode.segmentIndex === segmentIndex;
	const [optimisticBranchName, setOptimisticBranchName] = useOptimistic(
		branchName,
		(_currentBranchName, nextBranchName: string) => nextBranchName,
	);
	const [isRenamePending, startRenameTransition] = useTransition();

	const updateBranchName = useMutation(updateBranchNameMutationOptions);

	const endEditing = () => {
		runCommand(WorkspaceCommand.CancelMode());
		runCommand(WorkspaceCommand.SelectItem({ item }));
	};

	const saveBranchName = (newBranchName: string) => {
		if (branchName === null) return;
		const trimmed = newBranchName.trim();
		if (trimmed === "" || trimmed === branchName) return;
		startRenameTransition(async () => {
			setOptimisticBranchName(trimmed);
			try {
				await updateBranchName.mutateAsync({
					projectId,
					stackId,
					branchName,
					newName: trimmed,
				});
			} catch {
				// Use the global mutation error handler (shows toast) instead of React
				// error boundaries.
				return;
			}
			runCommand(
				WorkspaceCommand.SelectItem({
					item: segmentItem({
						stackId,
						segmentIndex,
						// TODO: ideally the API would return the new ref?
						branchRef: encodeRefName(`refs/heads/${trimmed}`),
					}),
				}),
			);
			runCommand(WorkspaceCommand.CancelMode());
		});
	};

	return (
		<OperationTarget
			{...restProps}
			projectId={projectId}
			item={item}
			operationMode={operationMode}
			selectedItem={selected ? item : null}
			render={
				<OperationSourceC
					operationMode={operationMode}
					projectId={projectId}
					source={operationSourceFromItem(item)}
					render={
						<ItemRow
							inert={!navigationIndexIncludes(navigationIndex, item)}
							isSelected={selected !== null}
						>
							{isRenaming && optimisticBranchName !== null ? (
								<InlineBranchNameEditor
									branchName={optimisticBranchName}
									formRef={branchRenameFormRef}
									onSubmit={saveBranchName}
									onExit={endEditing}
								/>
							) : (
								<>
									<ContextMenu.Root disabled={workspaceMode._tag !== "Default"}>
										<ContextMenu.Trigger
											render={
												<button
													type="button"
													className={styles.segmentButton}
													onClick={() => runCommand(WorkspaceCommand.SelectItem({ item }))}
												>
													{optimisticBranchName ?? "Untitled"}
												</button>
											}
										/>
										<ContextMenu.Portal>
											<ContextMenu.Positioner>
												<SegmentMenuPopup
													item={segmentItemV}
													canRename={branchName !== null && !isRenamePending}
													parts={ContextMenu}
												/>
											</ContextMenu.Positioner>
										</ContextMenu.Portal>
									</ContextMenu.Root>
									{workspaceMode._tag === "Default" && (
										<>
											<button
												type="button"
												className={sharedStyles.itemRowAction}
												aria-label="Push branch"
												disabled
											>
												<PushIcon />
											</button>
											<Menu.Root>
												<Menu.Trigger
													className={sharedStyles.itemRowAction}
													aria-label="Branch menu"
												>
													<MenuTriggerIcon />
												</Menu.Trigger>
												<Menu.Portal>
													<Menu.Positioner align="end">
														<SegmentMenuPopup
															item={segmentItemV}
															canRename={branchName !== null && !isRenamePending}
															parts={Menu}
														/>
													</Menu.Positioner>
												</Menu.Portal>
											</Menu.Root>
										</>
									)}
								</>
							)}
						</ItemRow>
					}
				/>
			}
		/>
	);
};

const SegmentC: FC<{
	branchRenameFormRef: Ref<HTMLFormElement>;
	commitMessageFormRef: Ref<HTMLFormElement>;
	operationMode: OperationMode | null;
	projectId: string;
	selectedItem: Item | null;
	segment: Segment;
	segmentIndex: number;
	stackId: string;
	workspaceMode: WorkspaceMode;
	navigationIndex: NavigationIndex;
}> = ({
	branchRenameFormRef,
	commitMessageFormRef,
	operationMode,
	projectId,
	segment,
	segmentIndex,
	selectedItem,
	stackId,
	workspaceMode,
	navigationIndex,
}) => {
	const selectedSegment =
		selectedItem?._tag === "Segment" &&
		selectedItem.stackId === stackId &&
		selectedItem.segmentIndex === segmentIndex
			? selectedItem
			: null;
	const selectedCommit =
		selectedItem?._tag === "Commit" &&
		selectedItem.stackId === stackId &&
		selectedItem.segmentIndex === segmentIndex
			? selectedItem
			: null;
	const selectedCommitFile =
		selectedItem?._tag === "CommitFile" &&
		selectedItem.stackId === stackId &&
		selectedItem.segmentIndex === segmentIndex
			? selectedItem
			: null;
	const isSectionSelected =
		selectedSegment !== null || selectedCommit !== null || selectedCommitFile !== null;

	return (
		<div
			className={classes(
				sharedStyles.section,
				styles.segment,
				isSectionSelected && sharedStyles.sectionSelected,
			)}
		>
			<SegmentRow
				branchRenameFormRef={branchRenameFormRef}
				operationMode={operationMode}
				selected={selectedSegment}
				workspaceMode={workspaceMode}
				projectId={projectId}
				segment={segment}
				stackId={stackId}
				segmentIndex={segmentIndex}
				navigationIndex={navigationIndex}
			/>

			<CommitsList commits={segment.commits}>
				{(commit) => {
					const isSelected = selectedCommit?.commitId === commit.id;
					return (
						<CommitC
							branchRef={segment.refName?.fullNameBytes ?? null}
							commit={commit}
							commitMessageFormRef={commitMessageFormRef}
							operationMode={operationMode}
							workspaceMode={workspaceMode}
							selected={isSelected ? selectedCommit : null}
							selectedFile={selectedCommitFile?.commitId === commit.id ? selectedCommitFile : null}
							projectId={projectId}
							segmentIndex={segmentIndex}
							stackId={stackId}
							navigationIndex={navigationIndex}
						/>
					);
				}}
			</CommitsList>
		</div>
	);
};

const StackC: FC<{
	branchRenameFormRef: Ref<HTMLFormElement>;
	commitMessageFormRef: Ref<HTMLFormElement>;
	operationMode: OperationMode | null;
	projectId: string;
	selectedItem: Item | null;
	stack: Stack;
	workspaceMode: WorkspaceMode;
	navigationIndex: NavigationIndex;
}> = ({
	branchRenameFormRef,
	commitMessageFormRef,
	operationMode,
	projectId,
	selectedItem,
	stack,
	workspaceMode,
	navigationIndex,
}) => {
	// From Caleb:
	// > There shouldn't be a way within GitButler to end up with a stack without a
	//   StackId. Users can disrupt our matching against our metadata by playing
	//   with references, but we currently also try to patch it up at certain points
	//   so it probably isn't too common.
	// For now we'll treat this as non-nullable until we identify cases where it
	// could genuinely be null (assuming backend correctness).
	// oxlint-disable-next-line typescript/no-non-null-assertion -- [tag:stack-id-required]
	const stackId = stack.id!;

	return (
		<div className={styles.stack}>
			<div>
				<div className={styles.laneActions}>
					<Menu.Root>
						<Menu.Trigger className={styles.stackMenuTrigger} aria-label="Stack menu">
							<MenuTriggerIcon />
						</Menu.Trigger>
						<Menu.Portal>
							<Menu.Positioner align="end">
								<StackMenuPopup stackId={stackId} />
							</Menu.Positioner>
						</Menu.Portal>
					</Menu.Root>
				</div>
				<Changes
					label="Assigned changes"
					operationMode={operationMode}
					projectId={projectId}
					stackId={stack.id}
					isSelected={selectedItem?._tag === "ChangesSection" && selectedItem.stackId === stackId}
					selectedPath={
						selectedItem?._tag === "Change" && selectedItem.stackId === stackId
							? selectedItem.path
							: null
					}
					navigationIndex={navigationIndex}
				/>
			</div>

			<ul className={styles.segments}>
				{stack.segments.map((segment, segmentIndex) => (
					// oxlint-disable-next-line react/no-array-index-key -- It's all we have.
					<li key={segmentIndex}>
						<SegmentC
							branchRenameFormRef={branchRenameFormRef}
							commitMessageFormRef={commitMessageFormRef}
							operationMode={operationMode}
							projectId={projectId}
							segment={segment}
							segmentIndex={segmentIndex}
							selectedItem={selectedItem}
							stackId={stackId}
							workspaceMode={workspaceMode}
							navigationIndex={navigationIndex}
						/>
					</li>
				))}
			</ul>
		</div>
	);
};

const ProjectPage: FC = () => {
	const { id: projectId } = Route.useParams();

	const expandedCommitId = useAppSelector((state) =>
		selectProjectExpandedCommitId(state, projectId),
	);
	const layoutState = useAppSelector((state) => selectProjectLayoutState(state, projectId));
	const selectedItemState = useAppSelector((state) => selectProjectSelectedItem(state, projectId));
	const workspaceModeState = useAppSelector((state) =>
		selectProjectWorkspaceModeState(state, projectId),
	);

	const branchRenameFormRef = useRef<HTMLFormElement | null>(null);
	const commitMessageFormRef = useRef<HTMLFormElement | null>(null);
	const previewRef = useRef<PreviewImperativeHandle | null>(null);

	const { data: projects } = useSuspenseQuery(listProjectsQueryOptions());

	const project = projects.find((project) => project.id === projectId);

	// TODO: handle project not found error. or only run when project is not null? waterfall.
	const { data: headInfo } = useSuspenseQuery(headInfoQueryOptions(projectId));

	const commonBaseCommitId = getCommonBaseCommitId(headInfo);

	const workspaceOutline = useWorkspaceOutline({ projectId, expandedCommitId });

	const navigationIndexUnfiltered = buildNavigationIndex(workspaceOutline);

	const resolveOperationSource = useResolveOperationSource(projectId);

	const workspaceMode = normalizeWorkspaceMode({
		mode: workspaceModeState,
		navigationIndex: navigationIndexUnfiltered,
	});

	const operationMode = getOperationMode(workspaceMode);

	const navigationIndex = operationMode
		? filterNavigationIndex(navigationIndexUnfiltered, (item) =>
				isOperationModeSourceOrTarget({
					item,
					operationMode,
					resolvedOperationSource: resolveOperationSource(operationMode.source),
				}),
			)
		: navigationIndexUnfiltered;

	const selectedItem =
		selectedItemState && navigationIndexIncludes(navigationIndex, selectedItemState)
			? selectedItemState
			: getDefaultItem(navigationIndex);

	const shortcutScope = getScope({ selectedItem, layoutState, workspaceMode });
	const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);

	const {
		absorptionPlan,
		isAbsorbing,
		requestAbsorptionPlan,
		confirmAbsorption,
		clearAbsorptionPlan,
	} = useAbsorption(projectId);

	const runCommand = useRunWorkspaceCommand({
		branchRenameFormRef,
		commitMessageFormRef,
		navigationIndex,
		operationMode,
		previewRef,
		projectId,
		requestAbsorptionPlan,
	});

	useMonitorDraggedOperationSource({ projectId });

	const handleCommandPaletteKeyDown = useEffectEvent((event: KeyboardEvent) => {
		if (event.defaultPrevented) return;
		if (event.altKey || event.shiftKey) return;
		if (!(event.metaKey || event.ctrlKey)) return;
		if (event.key.toLowerCase() !== "k") return;

		event.preventDefault();
		setIsCommandPaletteOpen((open) => !open);
	});

	useEffect(() => {
		window.addEventListener("keydown", handleCommandPaletteKeyDown);

		return () => {
			window.removeEventListener("keydown", handleCommandPaletteKeyDown);
		};
	}, []);

	useWorkspaceShortcuts({
		runCommand,
		scope: isCommandPaletteOpen ? null : shortcutScope,
	});

	// TODO: dedupe
	if (!project) return <p>Project not found.</p>;

	return (
		<WorkspaceCommandRuntimeContext value={{ runCommand, scope: shortcutScope }}>
			<ProjectPreviewLayout
				projectId={projectId}
				preview={
					selectedItem && (
						<Suspense fallback={<div>Loading preview…</div>}>
							<Preview
								operationMode={operationMode}
								projectId={projectId}
								selectedItem={selectedItem}
								isFocused={getFocus(layoutState) === "preview"}
								ref={previewRef}
							/>
						</Suspense>
					)
				}
			>
				<div className={sharedStyles.lanes}>
					<div className={styles.unassignedChangesLane}>
						<div className={styles.laneActions}>
							<Menu.Root>
								<Menu.Trigger className={styles.stackMenuTrigger} aria-label="Menu" disabled>
									<MenuTriggerIcon />
								</Menu.Trigger>
							</Menu.Root>
						</div>

						<Changes
							label="Unassigned changes"
							operationMode={operationMode}
							projectId={projectId}
							stackId={null}
							isSelected={selectedItem?._tag === "ChangesSection" && selectedItem.stackId === null}
							selectedPath={
								selectedItem?._tag === "Change" && selectedItem.stackId === null
									? selectedItem.path
									: null
							}
							navigationIndex={navigationIndex}
						/>
					</div>

					<div className={styles.headInfo}>
						<div className={styles.stackLanes}>
							{headInfo.stacks.map((stack) => (
								<div key={stack.id} className={styles.stackLane}>
									<StackC
										branchRenameFormRef={branchRenameFormRef}
										commitMessageFormRef={commitMessageFormRef}
										operationMode={operationMode}
										projectId={project.id}
										selectedItem={selectedItem}
										stack={stack}
										workspaceMode={workspaceMode}
										navigationIndex={navigationIndex}
									/>
								</div>
							))}
						</div>

						<div className={styles.commonBaseCommitContainer}>
							<OperationTarget
								projectId={projectId}
								item={baseCommitItem}
								operationMode={operationMode}
								selectedItem={selectedItem?._tag === "BaseCommit" ? selectedItem : null}
								render={
									<BaseCommitRow
										commitId={commonBaseCommitId}
										isSelected={selectedItem?._tag === "BaseCommit"}
										navigationIndex={navigationIndex}
									/>
								}
							/>
						</div>
					</div>
				</div>

				<PositionedTopBarActions>
					<WorkspaceCommandButton
						command={WorkspaceCommand.TogglePreview()}
						type="button"
						className={uiStyles.button}
						aria-pressed={isPreviewPanelVisible(layoutState)}
					/>
					<WorkspaceCommandButton
						command={WorkspaceCommand.ToggleFullscreenPreview()}
						type="button"
						className={uiStyles.button}
						aria-pressed={layoutState.isFullscreenPreviewOpen}
					/>
				</PositionedTopBarActions>

				{shortcutScope && (
					<PositionedShortcutsBar label={shortcutScope.label} items={shortcutScope.bindings} />
				)}

				<WorkspaceCommandPalette
					open={isCommandPaletteOpen}
					onOpenChange={setIsCommandPaletteOpen}
				/>

				{operationMode && (
					<div className={styles.operationModePreview}>
						<OperationSourceLabel headInfo={headInfo} source={operationMode.source} />
					</div>
				)}

				{absorptionPlan !== null && (
					<AbsorptionDialog
						absorptionPlan={absorptionPlan}
						isPending={isAbsorbing}
						onConfirm={confirmAbsorption}
						onOpenChange={(open) => {
							if (!open) clearAbsorptionPlan();
						}}
					/>
				)}
			</ProjectPreviewLayout>
		</WorkspaceCommandRuntimeContext>
	);
};

export const Route = createRoute({
	getParentRoute: () => projectRoute,
	path: "workspace",
	component: ProjectPage,
});
