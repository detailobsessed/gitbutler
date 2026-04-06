import { error } from "@sveltejs/kit";
import type { LayoutLoad } from "./$types";

export const prerender = false;

// eslint-disable-next-line
export const load: LayoutLoad = async ({ parent }) => {
	const { pinnedProjectId } = await parent();
	if (!pinnedProjectId) {
		error(404, "No pinned project");
	}
	return {
		projectId: pinnedProjectId,
		projectPinned: true,
	};
};
