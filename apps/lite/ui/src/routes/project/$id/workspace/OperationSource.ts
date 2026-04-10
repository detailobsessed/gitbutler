import { FileParent } from "#ui/domain/FileParent.ts";
import { type HunkHeader } from "@gitbutler/but-sdk";
import { Data, Match } from "effect";
import { Item } from "./Item.ts";

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
export type OperationSource = Data.TaggedEnum<{
	BaseCommit: {};
	ChangesSection: ChangesSectionOperationSource;
	Commit: CommitOperationSource;
	File: FileOperationSource;
	Hunk: HunkOperationSource;
	Segment: SegmentOperationSource;
}>;

export const OperationSource = Data.taggedEnum<OperationSource>();

/** @public */
export const baseCommitOperationSource: OperationSource = OperationSource.BaseCommit();

const operationSourceIdentityKey = (operationSource: OperationSource): string =>
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
				OperationSource.File({
					parent: FileParent.ChangesSection({ stackId }),
					path,
				}),
			ChangesSection: ({ stackId }) => OperationSource.ChangesSection({ stackId }),
			Commit: ({ commitId }) => OperationSource.Commit({ commitId }),
			CommitFile: ({ commitId, path }) =>
				OperationSource.File({
					parent: FileParent.Commit({ commitId }),
					path,
				}),
			Segment: ({ branchRef }) => OperationSource.Segment({ branchRef }),
		}),
	);

export const operationSourceMatchesItem = (source: OperationSource, item: Item): boolean =>
	operationSourceEquals(source, operationSourceFromItem(item));
