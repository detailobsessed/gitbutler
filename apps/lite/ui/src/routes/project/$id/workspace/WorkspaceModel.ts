import {
	changesInWorktreeQueryOptions,
	commitDetailsWithLineStatsQueryOptions,
	headInfoQueryOptions,
} from "#ui/api/queries.ts";
import { Segment, type HunkAssignment, type RefInfo, type TreeChange } from "@gitbutler/but-sdk";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { type NonEmptyArray } from "effect/Array";
import { Item, baseCommitItem, itemIdentityKey } from "./Item.ts";
import { getRelative } from "../shared.tsx";

const hasAssignmentsForPath = ({
	assignments,
	stackId,
	path,
}: {
	assignments: Array<HunkAssignment>;
	stackId: string | null;
	path: string;
}): boolean =>
	assignments.some(
		(assignment) => (assignment.stackId ?? null) === stackId && assignment.path === path,
	);

type WorkspaceSection = {
	section: Item;
	children: Array<Item>;
};

type WorkspaceOutline = NonEmptyArray<WorkspaceSection>;

type BuildWorkspaceOutlineArgs = {
	headInfo: RefInfo;
	changes: Array<TreeChange>;
	assignments: Array<HunkAssignment>;
	expandedCommitId?: string | null;
	expandedCommitPaths?: Array<string>;
};

const buildWorkspaceOutline = ({
	headInfo,
	changes,
	assignments,
	expandedCommitId = null,
	expandedCommitPaths,
}: BuildWorkspaceOutlineArgs): WorkspaceOutline => {
	const changesSection = (stackId: string | null): WorkspaceSection => ({
		section: Item.ChangesSection({ stackId }),
		children: changes.flatMap((change) =>
			hasAssignmentsForPath({ assignments, stackId, path: change.path })
				? [Item.Change({ stackId, path: change.path })]
				: [],
		),
	});

	const segmentSection = (
		stackId: string,
		segmentIndex: number,
		segment: Segment,
	): WorkspaceSection => {
		const branchRef = segment.refName?.fullNameBytes ?? null;
		return {
			section: Item.Segment({ stackId, segmentIndex, branchRef }),
			children: segment.commits.flatMap((commit) => [
				Item.Commit({ stackId, segmentIndex, branchRef, commitId: commit.id }),
				...(commit.id === expandedCommitId
					? (expandedCommitPaths ?? []).map((path) =>
							Item.CommitFile({
								stackId,
								segmentIndex,
								branchRef,
								commitId: commit.id,
								path,
							}),
						)
					: []),
			]),
		};
	};

	const baseCommitSection: WorkspaceSection = {
		section: baseCommitItem,
		children: [],
	};

	return [
		changesSection(null),

		...headInfo.stacks.flatMap((stack) => {
			if (stack.id == null) return [];
			const stackId = stack.id;
			return [
				changesSection(stackId),
				...stack.segments.map((segment, segmentIndex) =>
					segmentSection(stackId, segmentIndex, segment),
				),
			];
		}),

		baseCommitSection,
	];
};

export const useWorkspaceOutline = ({
	projectId,
	expandedCommitId,
}: {
	projectId: string;
	expandedCommitId: string | null;
}) => {
	const { data: headInfo } = useSuspenseQuery(headInfoQueryOptions(projectId));
	const { data: worktreeChanges } = useSuspenseQuery(changesInWorktreeQueryOptions(projectId));
	const { data: expandedCommitDetails } = useQuery({
		...commitDetailsWithLineStatsQueryOptions({
			projectId,
			commitId: expandedCommitId ?? "",
		}),
		enabled: expandedCommitId !== null,
	});

	return buildWorkspaceOutline({
		headInfo,
		changes: worktreeChanges.changes,
		assignments: worktreeChanges.assignments,
		expandedCommitId,
		expandedCommitPaths: expandedCommitDetails?.changes.map((change) => change.path) ?? [],
	});
};

export type NavigationIndex = {
	items: Array<Item>;
	sectionStartIndexes: Array<number>;
	sectionIndexByItemIndex: Array<number>;
	indexByKey: Map<string, number>;
};

export const buildNavigationIndex = (outline: WorkspaceOutline): NavigationIndex => {
	const model: NavigationIndex = {
		items: [],
		sectionStartIndexes: [],
		sectionIndexByItemIndex: [],
		indexByKey: new Map<string, number>(),
	};

	const addItem = (item: Item, sectionIndex: number) => {
		model.indexByKey.set(itemIdentityKey(item), model.items.length);
		model.sectionIndexByItemIndex.push(sectionIndex);
		model.items.push(item);
	};

	for (const { section, children } of outline) {
		const sectionIndex = model.sectionStartIndexes.length;
		model.sectionStartIndexes.push(model.items.length);
		addItem(section, sectionIndex);

		for (const item of children) addItem(item, sectionIndex);
	}

	return model;
};

export const filterNavigationIndex = (
	index: NavigationIndex,
	predicate: (item: Item) => boolean,
): NavigationIndex => {
	const filteredIndex: NavigationIndex = {
		items: [],
		sectionStartIndexes: [],
		sectionIndexByItemIndex: [],
		indexByKey: new Map<string, number>(),
	};

	let currentSectionIndex = -1;
	let filteredSectionIndex = -1;

	for (const [itemIndex, item] of index.items.entries()) {
		if (!predicate(item)) continue;

		const sectionIndex = index.sectionIndexByItemIndex[itemIndex] ?? -1;
		if (sectionIndex !== currentSectionIndex) {
			// Preserve the original section grouping, even when the section header
			// itself is filtered out and the retained item is one of its children.
			currentSectionIndex = sectionIndex;
			filteredSectionIndex = filteredIndex.sectionStartIndexes.length;
			filteredIndex.sectionStartIndexes.push(filteredIndex.items.length);
		}

		filteredIndex.indexByKey.set(itemIdentityKey(item), filteredIndex.items.length);
		filteredIndex.sectionIndexByItemIndex.push(filteredSectionIndex);
		filteredIndex.items.push(item);
	}

	return filteredIndex;
};

export const getAdjacentItem = (
	index: NavigationIndex,
	selection: Item | null,
	offset: -1 | 1,
): Item | null => {
	if (!selection) return null;
	const currentIndex = index.indexByKey.get(itemIdentityKey(selection));
	if (currentIndex === undefined) return null;
	return getRelative(index.items, currentIndex, offset);
};

export const getAdjacentSection = (
	index: NavigationIndex,
	selection: Item | null,
	offset: -1 | 1,
): Item | null => {
	if (!selection) return null;
	const currentIndex = index.indexByKey.get(itemIdentityKey(selection));
	if (currentIndex === undefined) return null;
	const currentSectionIndex = index.sectionIndexByItemIndex[currentIndex] ?? -1;
	if (currentSectionIndex === -1) return null;
	const adjacentSectionStartIndex = getRelative(
		index.sectionStartIndexes,
		currentSectionIndex,
		offset,
	);
	if (adjacentSectionStartIndex === null) return null;
	return index.items[adjacentSectionStartIndex] ?? null;
};

export const navigationIndexIncludes = (navigationIndex: NavigationIndex, item: Item): boolean =>
	navigationIndex.indexByKey.has(itemIdentityKey(item));

export const getDefaultItem = (navigationIndex: NavigationIndex): Item | null =>
	navigationIndex.items[0] ?? null;
