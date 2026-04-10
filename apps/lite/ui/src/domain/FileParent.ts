import { Data } from "effect";

/** @public */
export type CommitFileParent = { commitId: string };
/** @public */
export type ChangesSectionFileParent = { stackId: string | null };

export type FileParent = Data.TaggedEnum<{
	Commit: CommitFileParent;
	ChangesSection: ChangesSectionFileParent;
}>;

export const FileParent = Data.taggedEnum<FileParent>();
