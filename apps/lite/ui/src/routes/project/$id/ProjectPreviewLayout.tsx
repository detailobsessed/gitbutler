import { Dialog } from "@base-ui/react";
import { FC, ReactNode, use, useState } from "react";
import { Group, Panel, Separator, useDefaultLayout } from "react-resizable-panels";
import { isPreviewPanelVisible, Panel as PanelType } from "#ui/routes/project/$id/state/layout.ts";
import {
	projectActions,
	selectProjectLayoutState,
} from "#ui/routes/project/$id/state/projectSlice.ts";
import { ShortcutsBarPortalContext } from "#ui/routes/project/$id/ShortcutsBar.tsx";
import { useAppDispatch, useAppSelector } from "#ui/state/hooks.ts";
import uiStyles from "#ui/ui.module.css";
import { WorkspaceCommand } from "./workspace/WorkspaceCommands.ts";
import { WorkspaceCommandButton } from "./workspace/WorkspaceCommandButton.tsx";
import sharedStyles from "./shared.module.css";

export const ProjectPreviewLayout: FC<{
	projectId: string;
	children: ReactNode;
	preview: ReactNode | null;
}> = ({ children, projectId, preview }) => {
	const dispatch = useAppDispatch();
	const layoutState = useAppSelector((state) => selectProjectLayoutState(state, projectId));

	const inheritedShortcutsBarPortalNode = use(ShortcutsBarPortalContext);
	const [dialogShortcutsBarPortalNode, setDialogShortcutsBarPortalNode] =
		useState<HTMLElement | null>(null);
	const panelIds: Array<PanelType> = isPreviewPanelVisible(layoutState)
		? ["primary", "preview"]
		: ["primary"];
	const { defaultLayout, onLayoutChanged } = useDefaultLayout({
		id: `project:${projectId}:layout`,
		panelIds,
	});

	return (
		<ShortcutsBarPortalContext
			value={
				layoutState.isFullscreenPreviewOpen
					? (dialogShortcutsBarPortalNode ?? inheritedShortcutsBarPortalNode)
					: inheritedShortcutsBarPortalNode
			}
		>
			<Group
				className={sharedStyles.pageWithPreview}
				defaultLayout={defaultLayout}
				onLayoutChange={onLayoutChanged}
			>
				<Panel
					id={"primary" satisfies PanelType}
					minSize={500}
					className={sharedStyles.primaryPanel}
				>
					{children}
				</Panel>
				{isPreviewPanelVisible(layoutState) && (
					<>
						<Separator className={sharedStyles.previewResizeHandle} />
						<Panel
							id={"preview" satisfies PanelType}
							minSize={300}
							defaultSize="30%"
							className={sharedStyles.previewPanel}
						>
							{
								// There can only be one user of the ref at a time.
								layoutState.isFullscreenPreviewOpen ? null : preview
							}
						</Panel>
					</>
				)}
			</Group>
			{layoutState.isFullscreenPreviewOpen && (
				<Dialog.Root
					open
					onOpenChange={(open) => {
						dispatch(
							open
								? projectActions.openFullscreenPreview({ projectId })
								: projectActions.closeFullscreenPreview({ projectId }),
						);
					}}
				>
					<Dialog.Portal>
						<Dialog.Popup aria-label="Preview" className={sharedStyles.previewDialogPopup}>
							<div className={sharedStyles.previewDialogBody}>
								<WorkspaceCommandButton
									command={WorkspaceCommand.ClosePreview()}
									type="button"
									className={uiStyles.button}
								/>
								{preview}
							</div>
							<footer ref={setDialogShortcutsBarPortalNode} />
						</Dialog.Popup>
					</Dialog.Portal>
				</Dialog.Root>
			)}
		</ShortcutsBarPortalContext>
	);
};
