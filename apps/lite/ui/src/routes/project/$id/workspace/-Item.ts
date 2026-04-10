import { Match } from "effect";

/** @public */
export type ChangesSectionItem = { stackId: string | null };
/** @public */
export type ChangeItem = ChangesSectionItem & { path: string };

/** @public */
export type SegmentItem = {
	stackId: string;
	segmentIndex: number;
	branchRef: Array<number> | null;
};
/** @public */
export type CommitItem = SegmentItem & { commitId: string };
/** @public */
export type CommitFileItem = CommitItem & { path: string };

/**
 * A selectable item in the primary panel.
 */
export type Item =
	| ({ _tag: "ChangesSection" } & ChangesSectionItem)
	| ({ _tag: "Change" } & ChangeItem)
	| ({ _tag: "Segment" } & SegmentItem)
	| ({ _tag: "Commit" } & CommitItem)
	| ({ _tag: "CommitFile" } & CommitFileItem)
	| { _tag: "BaseCommit" };

/** @public */
export const changesSectionItem = ({ stackId }: ChangesSectionItem): Item => ({
	_tag: "ChangesSection",
	stackId,
});

/** @public */
export const changeItem = ({ stackId, path }: ChangeItem): Item => ({
	_tag: "Change",
	stackId,
	path,
});

/** @public */
export const segmentItem = ({ stackId, segmentIndex, branchRef }: SegmentItem): Item => ({
	_tag: "Segment",
	stackId,
	segmentIndex,
	branchRef,
});

/** @public */
export const commitItem = ({ stackId, segmentIndex, branchRef, commitId }: CommitItem): Item => ({
	_tag: "Commit",
	stackId,
	segmentIndex,
	branchRef,
	commitId,
});

/** @public */
export const commitFileItem = ({
	stackId,
	segmentIndex,
	branchRef,
	commitId,
	path,
}: CommitFileItem): Item => ({
	_tag: "CommitFile",
	stackId,
	segmentIndex,
	branchRef,
	commitId,
	path,
});

/** @public */
export const baseCommitItem: Item = {
	_tag: "BaseCommit",
};

export const itemIdentityKey = (item: Item): string =>
	Match.value(item).pipe(
		Match.tagsExhaustive({
			ChangesSection: (item) => JSON.stringify(["ChangesSection", item.stackId]),
			Change: (item) => JSON.stringify(["Change", item.stackId, item.path]),
			Segment: (item) =>
				JSON.stringify(["Segment", item.stackId, item.segmentIndex, item.branchRef]),
			Commit: (item) => JSON.stringify(["Commit", item.stackId, item.segmentIndex, item.commitId]),
			CommitFile: (item) =>
				JSON.stringify(["CommitFile", item.stackId, item.segmentIndex, item.commitId, item.path]),
			BaseCommit: () => JSON.stringify(["BaseCommit"]),
		}),
	);

export const itemEquals = (a: Item, b: Item): boolean => itemIdentityKey(a) === itemIdentityKey(b);

export const getParentSection = (item: Item): Item | null =>
	Match.value(item).pipe(
		Match.tagsExhaustive({
			Commit: (item) =>
				segmentItem({
					stackId: item.stackId,
					segmentIndex: item.segmentIndex,
					branchRef: item.branchRef,
				}),
			CommitFile: (item) =>
				commitItem({
					stackId: item.stackId,
					segmentIndex: item.segmentIndex,
					branchRef: item.branchRef,
					commitId: item.commitId,
				}),
			Change: (item) => changesSectionItem({ stackId: item.stackId }),
			ChangesSection: () => null,
			BaseCommit: () => null,
			Segment: () => null,
		}),
	);
