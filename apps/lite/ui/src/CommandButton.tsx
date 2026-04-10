import { classes } from "#ui/classes.ts";
import { mergeProps, Tooltip, useRender } from "@base-ui/react";
import type { FC } from "react";
import { formatShortcutKeys } from "#ui/shortcuts.ts";
import uiStyles from "#ui/ui.module.css";

type CommandButtonProps = {
	label: string;
	shortcutKeys?: Array<string> | null;
} & useRender.ComponentProps<"button">;

export const CommandButton: FC<CommandButtonProps> = ({
	label,
	render,
	shortcutKeys = null,
	...props
}) => {
	const ariaLabel =
		shortcutKeys && shortcutKeys.length > 0
			? `${label} (${formatShortcutKeys(shortcutKeys)})`
			: label;
	const trigger = useRender({
		render,
		defaultTagName: "button",
		props: mergeProps<"button">({ "aria-label": ariaLabel }, props),
	});

	return (
		<Tooltip.Root
			// Prevent tooltip from continuing to show when mouse moves from one
			// selected item to another.
			// [tag:tooltip-disable-hoverable-popup]
			disableHoverablePopup
		>
			<Tooltip.Trigger render={trigger} />
			<Tooltip.Portal>
				<Tooltip.Positioner sideOffset={8}>
					<Tooltip.Popup className={classes(uiStyles.popup, uiStyles.tooltip)}>
						<span>{label}</span>
						{shortcutKeys && (
							<span className={uiStyles.shortcutKeys}> ({formatShortcutKeys(shortcutKeys)})</span>
						)}
					</Tooltip.Popup>
				</Tooltip.Positioner>
			</Tooltip.Portal>
		</Tooltip.Root>
	);
};
