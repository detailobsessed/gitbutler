export type Panel = "primary" | "preview";
export const orderedPanels: Array<Panel> = ["primary", "preview"];

export type ProjectLayoutState = {
	visiblePanels: Array<Panel>;
};

export const createInitialState = (): ProjectLayoutState => ({
	visiblePanels: [...orderedPanels],
});

export const initialState: ProjectLayoutState = createInitialState();

const isPanelVisible = (state: ProjectLayoutState, panel: Panel): boolean =>
	state.visiblePanels.includes(panel);

const showPanel = (state: ProjectLayoutState, panel: Panel) => {
	if (isPanelVisible(state, panel)) return;
	state.visiblePanels = orderedPanels.filter(
		(candidate) => candidate === panel || isPanelVisible(state, candidate),
	);
};

const hidePanel = (state: ProjectLayoutState, panel: Panel) => {
	if (!isPanelVisible(state, panel)) return;

	state.visiblePanels = state.visiblePanels.filter((candidate) => candidate !== panel);
};

export const closePreview = (state: ProjectLayoutState) => {
	hidePanel(state, "preview");
};

export const togglePreview = (state: ProjectLayoutState) => {
	if (isPanelVisible(state, "preview")) {
		hidePanel(state, "preview");
		return;
	}

	showPanel(state, "preview");
};

export const getVisiblePanels = (state: ProjectLayoutState): Array<Panel> => state.visiblePanels;

export const isPreviewPanelVisible = (state: ProjectLayoutState): boolean =>
	isPanelVisible(state, "preview");
