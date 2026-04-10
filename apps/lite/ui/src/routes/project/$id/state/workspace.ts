import { Item, type CommitItem, type SegmentItem } from "../workspace/Item.ts";
import { type OperationSource } from "../workspace/OperationSource.ts";
import { WorkspaceMode, defaultWorkspaceMode } from "../workspace/WorkspaceMode.ts";

export type WorkspaceSelectionState = {
	hunk: string | null;
	item: Item | null;
};

const createInitialWorkspaceSelectionState = (): WorkspaceSelectionState => ({
	hunk: null,
	item: null,
});

export type WorkspaceState = {
	expandedCommitId: string | null;
	highlightedCommitIds: Array<string>;
	mode: WorkspaceMode;
	selection: WorkspaceSelectionState;
};

export const createInitialState = (): WorkspaceState => ({
	expandedCommitId: null,
	highlightedCommitIds: [],
	mode: defaultWorkspaceMode,
	selection: createInitialWorkspaceSelectionState(),
});

export const initialState: WorkspaceState = createInitialState();

const normalizeModeForSelectedItem = (mode: WorkspaceMode, item: Item | null) =>
	(mode._tag === "RewordCommit" &&
		(item === null || item._tag !== "Commit" || item.commitId !== mode.commitId)) ||
	(mode._tag === "RenameBranch" &&
		(item === null ||
			item._tag !== "Segment" ||
			item.stackId !== mode.stackId ||
			item.segmentIndex !== mode.segmentIndex))
		? defaultWorkspaceMode
		: mode;

export const closeCommitFiles = (state: WorkspaceState, item: CommitItem) => {
	state.expandedCommitId = null;
	selectItem(state, Item.Commit(item));
};

export const enterMoveMode = (state: WorkspaceState, source: OperationSource) => {
	state.mode = WorkspaceMode.Move({ source });
};

export const enterRubMode = (state: WorkspaceState, source: OperationSource) => {
	state.mode = WorkspaceMode.Rub({ source });
};

export const exitMode = (state: WorkspaceState) => {
	state.mode = defaultWorkspaceMode;
};

export const openCommitFiles = (state: WorkspaceState, item: CommitItem) => {
	state.expandedCommitId = item.commitId;
	selectItem(state, Item.Commit(item));
};

export const selectHunk = (state: WorkspaceState, hunk: string | null) => {
	state.selection.hunk = hunk;
};

export const selectItem = (state: WorkspaceState, item: Item | null) => {
	state.selection.item = item;
	state.selection.hunk = null;
	state.mode = normalizeModeForSelectedItem(state.mode, item);
};

export const setExpandedCommitId = (state: WorkspaceState, commitId: string | null) => {
	state.expandedCommitId = commitId;
};

export const setHighlightedCommitIds = (state: WorkspaceState, commitIds: Array<string> | null) => {
	state.highlightedCommitIds = commitIds ?? [];
};

export const startRenameBranch = (state: WorkspaceState, item: SegmentItem) => {
	selectItem(state, Item.Segment(item));
	state.mode = WorkspaceMode.RenameBranch({
		stackId: item.stackId,
		segmentIndex: item.segmentIndex,
	});
};

export const startRewordCommit = (state: WorkspaceState, item: CommitItem) => {
	selectItem(state, Item.Commit(item));
	state.mode = WorkspaceMode.RewordCommit({ commitId: item.commitId });
};

export const toggleCommitFiles = (state: WorkspaceState, item: CommitItem) => {
	if (state.expandedCommitId === item.commitId) {
		closeCommitFiles(state, item);
		return;
	}

	openCommitFiles(state, item);
};

const selectSelection = (state: WorkspaceState): WorkspaceSelectionState => state.selection;

export const selectSelectedItem = (state: WorkspaceState): Item | null =>
	selectSelection(state).item;

export const selectSelectedHunk = (state: WorkspaceState): string | null =>
	selectSelection(state).hunk;

export const selectMode = (state: WorkspaceState): WorkspaceMode => state.mode;

export const selectExpandedCommitId = (state: WorkspaceState): string | null =>
	state.expandedCommitId;

export const selectHighlightedCommitIds = (state: WorkspaceState): Array<string> =>
	state.highlightedCommitIds;

export const normalizeSelectedHunk = ({
	hunkKeys,
	selectedHunk,
}: {
	hunkKeys: Array<string>;
	selectedHunk: string | null;
}): string | undefined => {
	if (selectedHunk !== null && hunkKeys.includes(selectedHunk)) return selectedHunk;
	return hunkKeys[0];
};
