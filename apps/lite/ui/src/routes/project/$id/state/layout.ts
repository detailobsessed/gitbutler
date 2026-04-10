import { Data } from "effect";

export type Panel = "primary" | "preview";

/** @public */
export type SplitPanelLayout = { focus: Panel };
export type PanelLayout = Data.TaggedEnum<{
	Primary: {};
	Split: SplitPanelLayout;
}>;

export const PanelLayout = Data.taggedEnum<PanelLayout>();

/** @public */
export const primaryPanelLayout: PanelLayout = PanelLayout.Primary();

export type ProjectLayoutState = {
	isFullscreenPreviewOpen: boolean;
	panelLayout: PanelLayout;
};

export const createInitialState = (): ProjectLayoutState => ({
	isFullscreenPreviewOpen: false,
	panelLayout: PanelLayout.Split({ focus: "primary" }),
});

export const initialState: ProjectLayoutState = createInitialState();

export const closeFullscreenPreview = (state: ProjectLayoutState) => {
	state.isFullscreenPreviewOpen = false;
};

export const closePreview = (state: ProjectLayoutState) => {
	if (state.isFullscreenPreviewOpen) {
		state.isFullscreenPreviewOpen = false;
		return;
	}

	state.panelLayout = primaryPanelLayout;
};

export const focusPrimary = (state: ProjectLayoutState) => {
	state.isFullscreenPreviewOpen = false;
	state.panelLayout =
		state.panelLayout._tag === "Primary"
			? state.panelLayout
			: PanelLayout.Split({ focus: "primary" });
};

export const focusPreview = (state: ProjectLayoutState) => {
	if (state.isFullscreenPreviewOpen) return;
	state.panelLayout = PanelLayout.Split({ focus: "preview" });
};

export const openFullscreenPreview = (state: ProjectLayoutState) => {
	state.isFullscreenPreviewOpen = true;
};

export const toggleFullscreenPreview = (state: ProjectLayoutState) => {
	state.isFullscreenPreviewOpen = !state.isFullscreenPreviewOpen;
};

export const togglePreview = (state: ProjectLayoutState) => {
	state.panelLayout =
		state.panelLayout._tag === "Primary"
			? PanelLayout.Split({ focus: "primary" })
			: primaryPanelLayout;
};

const getPanelFocus = (state: ProjectLayoutState): Panel =>
	state.panelLayout._tag === "Split" ? state.panelLayout.focus : "primary";

export const getFocus = (state: ProjectLayoutState): Panel =>
	state.isFullscreenPreviewOpen ? "preview" : getPanelFocus(state);

export const isPreviewPanelVisible = (state: ProjectLayoutState): boolean =>
	state.panelLayout._tag === "Split";
