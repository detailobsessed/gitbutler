import { assert } from "#ui/routes/project/$id/shared.tsx";
import { formatShortcutKeys } from "#ui/shortcuts.ts";
import uiStyles from "#ui/ui.module.css";
import { mergeProps } from "@base-ui/react";
import { type FC, use } from "react";
import { getWorkspaceCommandLabel, type WorkspaceCommand } from "./WorkspaceCommands.ts";
import { WorkspaceCommandRuntimeContext } from "./WorkspaceCommandRuntime.tsx";
import { findScopeBinding } from "./WorkspaceShortcuts.ts";

type WorkspaceCommandMenuItemProps = {
	command: WorkspaceCommand;
} & React.ComponentProps<"div">;

export const WorkspaceCommandMenuItem: FC<WorkspaceCommandMenuItemProps> = ({
	command,
	...props
}) => {
	const { runCommand, scope } = assert(use(WorkspaceCommandRuntimeContext));
	const label = getWorkspaceCommandLabel(command);
	const shortcutKeys = findScopeBinding(scope, command)?.keys ?? null;

	return (
		<div
			{...mergeProps<"div">(props, {
				onClick: () => {
					runCommand(command);
				},
			})}
		>
			<span>{label}</span>
			{shortcutKeys && (
				<span className={uiStyles.shortcutKeys}> ({formatShortcutKeys(shortcutKeys)})</span>
			)}
		</div>
	);
};
