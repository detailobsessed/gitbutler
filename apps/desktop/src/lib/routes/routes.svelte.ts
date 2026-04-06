import { page } from "$app/state";

// These fields are set by project-specific layouts (p/[projectId] and (pinned)).
// They are not part of App.PageData because they are not available on all pages —
// accessing them here is intentional since this module exists solely to abstract
// over the current routing context.
type ProjectPageData = { projectId?: string; projectPinned?: boolean };

function isUrl<T>(id: string, pinnedId?: string): T | undefined {
	if (page.route.id === id) {
		return page.params as T;
	}
	// SvelteKit includes route group names in route IDs, so the pinned workspace is
	// "/(pinned)/workspace", not "/workspace". The pinnedId arg carries the full group-prefixed ID.
	if (pinnedId && page.route.id === pinnedId) {
		return { projectId: (page.data as ProjectPageData).projectId, ...page.params } as T;
	}
}

function prefix(projectId: string): string {
	return (page.data as ProjectPageData).projectPinned ? "" : `/p/${projectId}`;
}

export function workspacePath(projectId: string) {
	return `${prefix(projectId)}/workspace`;
}

export function isWorkspacePath(): { projectId: string; stackId?: string } | undefined {
	const isWorkspaceUrl = isUrl<{ projectId: string }>(
		"/p/[projectId]/workspace",
		"/(pinned)/workspace",
	);
	if (!isWorkspaceUrl) return undefined;
	// stackId is a query param, not a route segment — page.route.id never includes query strings
	// and page.params never contains query parameters.
	const stackId = page.url.searchParams.get("stackId") ?? undefined;
	return { ...isWorkspaceUrl, stackId };
}

/** Navigates to the workspace for the given project (the default project view). */
export function projectPath(projectId: string) {
	return workspacePath(projectId);
}

export function isProjectPath() {
	return isWorkspacePath();
}

export function historyPath(projectId: string) {
	return `${prefix(projectId)}/history`;
}

export function isHistoryPath() {
	return isUrl<{ projectId: string }>("/p/[projectId]/history", "/(pinned)/history");
}

export function branchesPath(projectId: string) {
	return `${prefix(projectId)}/branches`;
}

export function isBranchesPath() {
	return isUrl<{ projectId: string }>("/p/[projectId]/branches", "/(pinned)/branches");
}

export function codegenPath(projectId: string) {
	return `${prefix(projectId)}/codegen`;
}

export function isCodegenPath() {
	return isUrl<{ projectId: string }>("/p/[projectId]/codegen", "/(pinned)/codegen");
}

export function isPreviewStackPath() {
	return isUrl<{ projectId: string }>("/p/[projectId]/preview-stack/[stackId]");
}

export function previewStackPath(projectId: string, stackId: string) {
	return `${prefix(projectId)}/preview-stack/${stackId}`;
}

export function isCommitPath() {
	return page.url.searchParams.has("create");
}

export function editModePath(projectId: string) {
	return `${prefix(projectId)}/edit`;
}

export function clonePath() {
	return "/onboarding/clone";
}
