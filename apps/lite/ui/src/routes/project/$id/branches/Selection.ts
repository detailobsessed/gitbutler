import { Data } from "effect";
import { BranchIdentity, BranchListing } from "@gitbutler/but-sdk";

/** @public */
export type BranchSelection = { branchName: BranchIdentity };
/** @public */
export type DetailsCommitMode = { path?: string };
/** @public */
export type CommitMode = Data.TaggedEnum<{
	Summary: {};
	Details: DetailsCommitMode;
}>;
/** @public */
export type CommitSelection = BranchSelection & { commitId: string; mode: CommitMode };
export type Selection = Data.TaggedEnum<{
	Branch: BranchSelection;
	Commit: CommitSelection;
}>;

export const CommitMode = Data.taggedEnum<CommitMode>();
export const Selection = Data.taggedEnum<Selection>();

export const isValidBranchSelection = (
	selection: Selection,
	branches: Array<BranchListing>,
): boolean => {
	const branch = branches.find((branch) => branch.name === selection.branchName);
	if (!branch) return false;
	return true;
};

export const getDefaultSelection = (branches: Array<BranchListing>): Selection | null => {
	const firstBranch = branches[0];
	if (!firstBranch) return null;
	return Selection.Branch({ branchName: firstBranch.name });
};
