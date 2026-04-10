import { CommandButton } from "#ui/CommandButton.tsx";
import { assert } from "#ui/routes/project/$id/shared.tsx";
import { type ComponentProps, FC, use } from "react";
import { getWorkspaceCommandLabel, type WorkspaceCommand } from "./WorkspaceCommands.ts";
import { WorkspaceCommandRuntimeContext } from "./WorkspaceCommandRuntime.tsx";
import { findScopeBinding } from "./WorkspaceShortcuts.ts";

type WorkspaceCommandButtonProps = {
	command: WorkspaceCommand;
} & Omit<ComponentProps<typeof CommandButton>, "label" | "shortcutKeys">;

export const WorkspaceCommandButton: FC<WorkspaceCommandButtonProps> = ({
	children,
	command,
	onClick,
	...props
}) => {
	const { runCommand, scope } = assert(use(WorkspaceCommandRuntimeContext));
	const label = getWorkspaceCommandLabel(command);
	const shortcutKeys = findScopeBinding(scope, command)?.keys ?? null;

	return (
		<CommandButton
			{...props}
			label={label}
			shortcutKeys={shortcutKeys}
			onClick={(event) => {
				onClick?.(event);
				if (event.defaultPrevented) return;
				runCommand(command);
			}}
		>
			{children ?? label}
		</CommandButton>
	);
};
