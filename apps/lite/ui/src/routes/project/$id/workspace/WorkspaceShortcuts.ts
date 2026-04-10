import { getAction, type ShortcutBinding } from "#ui/shortcuts.ts";
import { isTypingTarget } from "#ui/routes/project/$id/shared.tsx";
import { getFocus, type ProjectLayoutState } from "#ui/routes/project/$id/state/layout.ts";
import { Match } from "effect";
import { useEffect, useEffectEvent } from "react";
import {
	baseCommitItem,
	changeItem,
	changesSectionItem,
	commitFileItem,
	commitItem,
	type CommitFileItem,
	type CommitItem,
	type Item,
	type SegmentItem,
	segmentItem,
} from "./Item.ts";
import { operationSourceFromItem } from "./OperationSource.ts";
import {
	getWorkspaceCommandLabel,
	WorkspaceCommand,
	workspaceCommandEquals,
} from "./WorkspaceCommands.ts";
import type { WorkspaceMode } from "./WorkspaceMode.ts";

const withDescription = (
	x: Omit<ShortcutBinding<WorkspaceCommand>, "description">,
): ShortcutBinding<WorkspaceCommand> => ({
	...x,
	description: getWorkspaceCommandLabel(x.action),
});

export type Scope = {
	allowWhenTyping: boolean;
	bindings: Array<ShortcutBinding<WorkspaceCommand>>;
	label: string;
};

const togglePreviewBinding = withDescription({
	id: "toggle-preview",
	keys: ["p"],
	action: WorkspaceCommand.TogglePreview(),
	repeat: false,
});

const toggleFullscreenPreviewBinding = withDescription({
	id: "toggle-fullscreen-preview",
	keys: ["d"],
	action: WorkspaceCommand.ToggleFullscreenPreview(),
	repeat: false,
});

const getItemSelectionBindings = (selectedItem: Item): Array<ShortcutBinding<WorkspaceCommand>> => [
	withDescription({
		id: "item-selection-move-up",
		keys: ["ArrowUp", "k"],
		action: WorkspaceCommand.SelectRelativeItem({ item: selectedItem, offset: -1 }),
	}),
	withDescription({
		id: "item-selection-move-down",
		keys: ["ArrowDown", "j"],
		action: WorkspaceCommand.SelectRelativeItem({ item: selectedItem, offset: 1 }),
	}),
	withDescription({
		id: "item-selection-previous-section",
		keys: ["Shift+ArrowUp", "Shift+k"],
		action: WorkspaceCommand.SelectPreviousSection({ item: selectedItem }),
		showInShortcutsBar: false,
	}),
	withDescription({
		id: "item-selection-next-section",
		keys: ["Shift+ArrowDown", "Shift+j"],
		action: WorkspaceCommand.SelectNextSection({ item: selectedItem }),
		showInShortcutsBar: false,
	}),
	withDescription({
		id: "item-selection-enter-rub-mode",
		keys: ["r"],
		action: WorkspaceCommand.EnterRubMode({
			source: operationSourceFromItem(selectedItem),
		}),
		repeat: false,
	}),
	withDescription({
		id: "item-selection-enter-move-mode",
		keys: ["m"],
		action: WorkspaceCommand.EnterMoveMode({
			source: operationSourceFromItem(selectedItem),
		}),
		repeat: false,
	}),
];

const getPrimaryPanelBindings = (selectedItem: Item): Array<ShortcutBinding<WorkspaceCommand>> => [
	...getItemSelectionBindings(selectedItem),
	withDescription({
		id: "primary-panel-select-unassigned-changes",
		keys: ["z"],
		action: WorkspaceCommand.SelectUnassignedChanges(),
		repeat: false,
	}),
	withDescription({
		id: "primary-panel-focus-preview",
		keys: ["l"],
		action: WorkspaceCommand.FocusPreview(),
		repeat: false,
	}),
	toggleFullscreenPreviewBinding,
	togglePreviewBinding,
];

const getChangesBindings = (selectedItem: Item): Array<ShortcutBinding<WorkspaceCommand>> => [
	...getPrimaryPanelBindings(selectedItem),
	withDescription({
		id: "changes-absorb",
		keys: ["a"],
		action: WorkspaceCommand.Absorb({ item: selectedItem }),
		repeat: false,
	}),
];

const getCommitBindings = (selectedItem: CommitItem): Array<ShortcutBinding<WorkspaceCommand>> => [
	...getPrimaryPanelBindings(commitItem(selectedItem)),
	withDescription({
		id: "commit-toggle-files",
		keys: ["f"],
		action: WorkspaceCommand.ToggleCommitFiles({ item: selectedItem }),
		repeat: false,
	}),
	withDescription({
		id: "commit-reword",
		keys: ["Enter"],
		action: WorkspaceCommand.StartRewordCommit({ item: selectedItem }),
		repeat: false,
	}),
];

const getCommitFileBindings = (
	selectedItem: CommitFileItem,
): Array<ShortcutBinding<WorkspaceCommand>> => [
	...getPrimaryPanelBindings(commitFileItem(selectedItem)),
	withDescription({
		id: "commit-toggle-files",
		keys: ["f"],
		action: WorkspaceCommand.ToggleCommitFiles({ item: selectedItem }),
		repeat: false,
	}),
	withDescription({
		id: "commit-file-close",
		keys: ["Escape"],
		action: WorkspaceCommand.CloseCommitFiles({ item: selectedItem }),
		repeat: false,
	}),
];

const getBranchBindings = (selectedItem: SegmentItem): Array<ShortcutBinding<WorkspaceCommand>> => [
	...getPrimaryPanelBindings(segmentItem(selectedItem)),
	withDescription({
		id: "branch-rename",
		keys: ["Enter"],
		action: WorkspaceCommand.StartRenameBranch({ item: selectedItem }),
		repeat: false,
	}),
];

const previewBindings: Array<ShortcutBinding<WorkspaceCommand>> = [
	withDescription({
		id: "preview-move-up",
		keys: ["ArrowUp", "k"],
		action: WorkspaceCommand.MovePreviewSelection({ offset: -1 }),
	}),
	withDescription({
		id: "preview-move-down",
		keys: ["ArrowDown", "j"],
		action: WorkspaceCommand.MovePreviewSelection({ offset: 1 }),
	}),
	withDescription({
		id: "preview-focus-primary",
		keys: ["h"],
		action: WorkspaceCommand.FocusPrimary(),
		repeat: false,
	}),
	toggleFullscreenPreviewBinding,
	togglePreviewBinding,
	withDescription({
		id: "preview-close",
		keys: ["Escape"],
		action: WorkspaceCommand.ClosePreview(),
		repeat: false,
	}),
];

const fullscreenPreviewBindings: Array<ShortcutBinding<WorkspaceCommand>> = previewBindings.filter(
	(binding) => binding.action._tag !== "TogglePreview",
);

const getOperationModeBindings = (
	selectedItem: Item | null,
): Array<ShortcutBinding<WorkspaceCommand>> => [
	...(selectedItem
		? getPrimaryPanelBindings(selectedItem).filter(
				(binding) =>
					binding.action._tag !== "EnterRubMode" && binding.action._tag !== "EnterMoveMode",
			)
		: [
				withDescription({
					id: "primary-panel-select-unassigned-changes",
					keys: ["z"],
					action: WorkspaceCommand.SelectUnassignedChanges(),
					repeat: false,
				}),
				withDescription({
					id: "primary-panel-focus-preview",
					keys: ["l"],
					action: WorkspaceCommand.FocusPreview(),
					repeat: false,
				}),
				toggleFullscreenPreviewBinding,
				togglePreviewBinding,
			]),
	withDescription({
		id: "operation-mode-confirm",
		keys: ["Enter"],
		action: WorkspaceCommand.RunOperation({ target: selectedItem }),
		repeat: false,
	}),
	withDescription({
		id: "operation-mode-cancel",
		keys: ["Escape"],
		action: WorkspaceCommand.CancelMode(),
		repeat: false,
	}),
];

const rewordCommitBindings: Array<ShortcutBinding<WorkspaceCommand>> = [
	withDescription({
		id: "commit-reword-save",
		keys: ["Enter"],
		action: WorkspaceCommand.SubmitCommitMessage(),
		repeat: false,
	}),
	withDescription({
		id: "commit-reword-cancel",
		keys: ["Escape"],
		action: WorkspaceCommand.CancelMode(),
		repeat: false,
	}),
];

const renameBranchBindings: Array<ShortcutBinding<WorkspaceCommand>> = [
	withDescription({
		id: "branch-rename-save",
		keys: ["Enter"],
		action: WorkspaceCommand.SubmitBranchRename(),
		repeat: false,
	}),
	withDescription({
		id: "branch-rename-cancel",
		keys: ["Escape"],
		action: WorkspaceCommand.CancelMode(),
		repeat: false,
	}),
];

const getDefaultModeScope = (selectedItem: Item): Scope =>
	Match.value(selectedItem).pipe(
		Match.tagsExhaustive({
			BaseCommit: (): Scope => ({
				allowWhenTyping: false,
				label: "Base commit",
				bindings: getPrimaryPanelBindings(baseCommitItem),
			}),
			Change: (selectedItem): Scope => ({
				allowWhenTyping: false,
				label: "Change",
				bindings: getChangesBindings(changeItem(selectedItem)),
			}),
			ChangesSection: (selectedItem): Scope => ({
				allowWhenTyping: false,
				label: "Changes",
				bindings: getChangesBindings(changesSectionItem(selectedItem)),
			}),
			Commit: (selectedItem): Scope => ({
				allowWhenTyping: false,
				label: "Commit",
				bindings: getCommitBindings(selectedItem),
			}),
			CommitFile: (selectedItem): Scope => ({
				allowWhenTyping: false,
				label: "Commit file",
				bindings: getCommitFileBindings(selectedItem),
			}),
			Segment: (selectedItem): Scope => ({
				allowWhenTyping: false,
				label: selectedItem.branchRef === null ? "Segment" : "Branch",
				bindings:
					selectedItem.branchRef === null
						? getPrimaryPanelBindings(segmentItem(selectedItem))
						: getBranchBindings(selectedItem),
			}),
		}),
	);

const getModeScope = ({
	selectedItem,
	workspaceMode,
}: {
	selectedItem: Item | null;
	workspaceMode: WorkspaceMode;
}): Scope | null =>
	Match.value(workspaceMode).pipe(
		Match.tagsExhaustive({
			Default: () => (selectedItem ? getDefaultModeScope(selectedItem) : null),
			Move: (): Scope | null => ({
				allowWhenTyping: false,
				label: "Move mode",
				bindings: getOperationModeBindings(selectedItem),
			}),
			RenameBranch: (workspaceMode): Scope | null =>
				selectedItem?._tag === "Segment" &&
				workspaceMode.stackId === selectedItem.stackId &&
				workspaceMode.segmentIndex === selectedItem.segmentIndex
					? {
							label: "Rename branch",
							bindings: renameBranchBindings,
							allowWhenTyping: true,
						}
					: null,
			RewordCommit: (workspaceMode): Scope | null =>
				selectedItem?._tag === "Commit" && workspaceMode.commitId === selectedItem.commitId
					? {
							label: "Reword commit",
							bindings: rewordCommitBindings,
							allowWhenTyping: true,
						}
					: null,
			Rub: (): Scope | null => ({
				allowWhenTyping: false,
				label: "Rub mode",
				bindings: getOperationModeBindings(selectedItem),
			}),
		}),
	);

export const getScope = ({
	selectedItem,
	layoutState,
	workspaceMode,
}: {
	selectedItem: Item | null;
	layoutState: ProjectLayoutState;
	workspaceMode: WorkspaceMode;
}): Scope | null => {
	if (getFocus(layoutState) === "preview")
		return {
			allowWhenTyping: false,
			label: "Preview",
			bindings: layoutState.isFullscreenPreviewOpen ? fullscreenPreviewBindings : previewBindings,
		};

	return getModeScope({ selectedItem, workspaceMode });
};

export const findScopeBinding = (
	scope: Scope | null,
	command: WorkspaceCommand,
): ShortcutBinding<WorkspaceCommand> | null => {
	const binding = scope?.bindings.find((binding) =>
		workspaceCommandEquals(binding.action, command),
	);
	return binding ?? null;
};

export const useWorkspaceShortcuts = ({
	runCommand,
	scope,
}: {
	runCommand: (command: WorkspaceCommand) => void;
	scope: Scope | null;
}) => {
	const handleKeyDown = useEffectEvent((event: KeyboardEvent) => {
		if (event.defaultPrevented) return;
		if (!scope) return;
		if (!scope.allowWhenTyping && isTypingTarget(event.target)) return;

		const command = getAction(scope.bindings, event);
		if (!command) return;

		event.preventDefault();
		runCommand(command);
	});

	useEffect(() => {
		window.addEventListener("keydown", handleKeyDown);

		return () => {
			window.removeEventListener("keydown", handleKeyDown);
		};
	}, []);
};
