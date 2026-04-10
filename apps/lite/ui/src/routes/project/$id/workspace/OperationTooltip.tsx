import { classes } from "#ui/classes.ts";
import { operationLabel, type Operation } from "#ui/Operation.ts";
import uiStyles from "#ui/ui.module.css";
import { Tooltip, useRender } from "@base-ui/react";
import { FC } from "react";
import styles from "./route.module.css";
import { OperationSource, operationSourceMatchesItem } from "./OperationSource";
import { Item } from "./Item";
import { WorkspaceCommandButton } from "./WorkspaceCommandButton.tsx";
import { WorkspaceCommand } from "./WorkspaceCommands.ts";

export const OperationTooltip: FC<
	{
		enabled: boolean;
		operation: Operation | null;
		sourceOperation?: OperationSource;
		sourceItem: Item;
		controls: boolean;
	} & useRender.ComponentProps<"div">
> = ({
	enabled,
	operation,
	sourceOperation: source,
	sourceItem: item,
	controls,
	render,
	...props
}) => {
	const isSource = source && operationSourceMatchesItem(source, item);

	const tooltip = enabled ? (
		isSource ? (
			<>Select a target</>
		) : operation ? (
			controls ? (
				<>
					<span>{operationLabel(operation)}</span>
					<WorkspaceCommandButton
						command={WorkspaceCommand.RunOperation({ target: item })}
						type="button"
						className={uiStyles.button}
					/>
					<WorkspaceCommandButton
						command={WorkspaceCommand.CancelMode()}
						type="button"
						className={uiStyles.button}
					/>
				</>
			) : (
				<>{operationLabel(operation)}</>
			)
		) : null
	) : null;

	const trigger = useRender({ render, props });

	return (
		<Tooltip.Root
			open={!!tooltip}
			disableHoverablePopup={!controls}
			onOpenChange={(_open, eventDetails) => {
				eventDetails.allowPropagation();
			}}
		>
			<Tooltip.Trigger render={trigger} />
			<Tooltip.Portal>
				<Tooltip.Positioner sideOffset={8}>
					<Tooltip.Popup
						className={classes(uiStyles.popup, uiStyles.tooltip, styles.operationTooltipPopup)}
					>
						{tooltip}
					</Tooltip.Popup>
				</Tooltip.Positioner>
			</Tooltip.Portal>
		</Tooltip.Root>
	);
};
