import { createContext } from "react";
import type { WorkspaceCommand } from "./WorkspaceCommands.ts";
import type { Scope } from "./WorkspaceShortcuts.ts";

type WorkspaceCommandRuntime = {
	runCommand: (command: WorkspaceCommand) => void;
	scope: Scope | null;
};

export const WorkspaceCommandRuntimeContext = createContext<WorkspaceCommandRuntime | null>(null);
