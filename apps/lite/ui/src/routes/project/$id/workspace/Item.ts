import { Data, Match } from "effect";

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
export type Item = Data.TaggedEnum<{
	ChangesSection: ChangesSectionItem;
	Change: ChangeItem;
	Segment: SegmentItem;
	Commit: CommitItem;
	CommitFile: CommitFileItem;
	BaseCommit: {};
}>;

export const Item = Data.taggedEnum<Item>();

/** @public */
export const baseCommitItem: Item = Item.BaseCommit();

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
				Item.Segment({
					stackId: item.stackId,
					segmentIndex: item.segmentIndex,
					branchRef: item.branchRef,
				}),
			CommitFile: (item) =>
				Item.Commit({
					stackId: item.stackId,
					segmentIndex: item.segmentIndex,
					branchRef: item.branchRef,
					commitId: item.commitId,
				}),
			Change: (item) => Item.ChangesSection({ stackId: item.stackId }),
			ChangesSection: () => null,
			BaseCommit: () => null,
			Segment: () => null,
		}),
	);
