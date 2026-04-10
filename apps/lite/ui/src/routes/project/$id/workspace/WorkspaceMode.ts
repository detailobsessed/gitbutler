import { Data, Match } from "effect";
import { type OperationSource, operationSourceMatchesItem } from "./OperationSource.ts";
import { type NavigationIndex } from "./WorkspaceModel.ts";

/** @public */
export type RubOperationMode = { source: OperationSource };
/** @public */
export type MoveOperationMode = { source: OperationSource };

/** @public */
export type RewordCommitWorkspaceMode = { commitId: string };
/** @public */
export type RenameBranchWorkspaceMode = { stackId: string; segmentIndex: number };
export type WorkspaceMode = Data.TaggedEnum<{
	Default: {};
	RewordCommit: RewordCommitWorkspaceMode;
	RenameBranch: RenameBranchWorkspaceMode;
	Rub: RubOperationMode;
	Move: MoveOperationMode;
}>;
export type OperationMode = Extract<WorkspaceMode, { _tag: "Rub" | "Move" }>;

export const WorkspaceMode = Data.taggedEnum<WorkspaceMode>();

/** @public */
export const defaultWorkspaceMode: WorkspaceMode = WorkspaceMode.Default();

export const getOperationMode = (mode: WorkspaceMode): OperationMode | null =>
	mode._tag === "Rub" || mode._tag === "Move" ? mode : null;

export const normalizeWorkspaceMode = ({
	mode,
	navigationIndex,
}: {
	mode: WorkspaceMode;
	navigationIndex: NavigationIndex;
}): WorkspaceMode =>
	Match.value(mode).pipe(
		Match.tagsExhaustive({
			Default: () => mode,
			Rub: (mode) =>
				navigationIndex.items.some((item) => operationSourceMatchesItem(mode.source, item))
					? mode
					: defaultWorkspaceMode,
			Move: (mode) =>
				navigationIndex.items.some((item) => operationSourceMatchesItem(mode.source, item))
					? mode
					: defaultWorkspaceMode,
			RewordCommit: (mode) =>
				navigationIndex.items.some(
					(item) => item._tag === "Commit" && item.commitId === mode.commitId,
				)
					? mode
					: defaultWorkspaceMode,
			RenameBranch: (mode) =>
				navigationIndex.items.some(
					(item) =>
						item._tag === "Segment" &&
						item.stackId === mode.stackId &&
						item.segmentIndex === mode.segmentIndex,
				)
					? mode
					: defaultWorkspaceMode,
		}),
	);
