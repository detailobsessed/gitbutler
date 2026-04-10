import {
	changesSectionFileParent,
	commitFileParent,
	type FileParent,
} from "#ui/domain/FileParent.ts";
import { type HunkHeader } from "@gitbutler/but-sdk";
import { Match } from "effect";
import { type Item } from "./Item.ts";

/** @public */
export type ChangesSectionOperationSource = { stackId: string | null };
/** @public */
export type CommitOperationSource = { commitId: string };
/** @public */
export type FileOperationSource = { parent: FileParent; path: string };
/** @public */
export type HunkOperationSource = { parent: FileParent; path: string; hunkHeader: HunkHeader };
/** @public */
export type SegmentOperationSource = { branchRef: Array<number> | null };

/**
 * The source of an operation before it has been materialized into data that can
 * be sent to the backend (`ResolvedOperationSource`).
 */
export type OperationSource =
	| { _tag: "BaseCommit" }
	| ({ _tag: "ChangesSection" } & ChangesSectionOperationSource)
	| ({ _tag: "Commit" } & CommitOperationSource)
	| ({ _tag: "File" } & FileOperationSource)
	| ({ _tag: "Hunk" } & HunkOperationSource)
	| ({ _tag: "Segment" } & SegmentOperationSource);

/** @public */
export const baseCommitOperationSource: OperationSource = {
	_tag: "BaseCommit",
};

/** @public */
export const changesSectionOperationSource = ({
	stackId,
}: ChangesSectionOperationSource): OperationSource => ({
	_tag: "ChangesSection",
	stackId,
});

/** @public */
export const commitOperationSource = ({ commitId }: CommitOperationSource): OperationSource => ({
	_tag: "Commit",
	commitId,
});

/** @public */
export const fileOperationSource = ({ parent, path }: FileOperationSource): OperationSource => ({
	_tag: "File",
	parent,
	path,
});

/** @public */
export const hunkOperationSource = ({
	parent,
	path,
	hunkHeader,
}: HunkOperationSource): OperationSource => ({
	_tag: "Hunk",
	parent,
	path,
	hunkHeader,
});

/** @public */
export const segmentOperationSource = ({ branchRef }: SegmentOperationSource): OperationSource => ({
	_tag: "Segment",
	branchRef,
});

export const operationSourceIdentityKey = (operationSource: OperationSource): string =>
	Match.value(operationSource).pipe(
		Match.tagsExhaustive({
			BaseCommit: () => JSON.stringify(["BaseCommit"]),
			ChangesSection: ({ stackId }) => JSON.stringify(["ChangesSection", stackId]),
			Commit: ({ commitId }) => JSON.stringify(["Commit", commitId]),
			File: ({ parent, path }) => JSON.stringify(["File", parent, path]),
			Hunk: ({ parent, path, hunkHeader }) => JSON.stringify(["Hunk", parent, path, hunkHeader]),
			Segment: ({ branchRef }) => JSON.stringify(["Segment", branchRef]),
		}),
	);

export const operationSourceEquals = (a: OperationSource, b: OperationSource): boolean =>
	operationSourceIdentityKey(a) === operationSourceIdentityKey(b);

export const operationSourceFromItem = (item: Item): OperationSource =>
	Match.value(item).pipe(
		Match.tagsExhaustive({
			BaseCommit: () => baseCommitOperationSource,
			Change: ({ stackId, path }) =>
				fileOperationSource({
					parent: changesSectionFileParent({ stackId }),
					path,
				}),
			ChangesSection: ({ stackId }) => changesSectionOperationSource({ stackId }),
			Commit: ({ commitId }) => commitOperationSource({ commitId }),
			CommitFile: ({ commitId, path }) =>
				fileOperationSource({
					parent: commitFileParent({ commitId }),
					path,
				}),
			Segment: ({ branchRef }) => segmentOperationSource({ branchRef }),
		}),
	);

export const operationSourceMatchesItem = (source: OperationSource, item: Item): boolean =>
	operationSourceEquals(source, operationSourceFromItem(item));
