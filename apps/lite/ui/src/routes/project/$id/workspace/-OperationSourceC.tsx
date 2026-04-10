import { headInfoQueryOptions } from "#ui/api/queries.ts";
import { classes } from "#ui/classes.ts";
import { useDraggable } from "#ui/hooks/useDraggable.tsx";
import { mergeProps, useRender } from "@base-ui/react";
import { useSuspenseQuery } from "@tanstack/react-query";
import { FC } from "react";
import { DragData, DragPreview } from "./-DragAndDrop.tsx";
import { OperationSourceLabel } from "./-OperationSourceLabel.tsx";
import { operationSourceEquals, type OperationSource } from "./-OperationSource.ts";
import { type OperationMode } from "./-WorkspaceMode.ts";
import styles from "./route.module.css";

export const OperationSourceC: FC<
	{
		operationMode?: OperationMode | null;
		projectId: string;
		source: OperationSource;
		canDrag?: () => boolean;
	} & useRender.ComponentProps<"div">
> = ({ operationMode = null, projectId, source, canDrag, render, ...props }) => {
	const { data: headInfo } = useSuspenseQuery(headInfoQueryOptions(projectId));

	const [isDragging, dragRef] = useDraggable({
		getInitialData: (): DragData => ({ operationSource: source }),
		preview: (
			<DragPreview>
				<OperationSourceLabel source={source} headInfo={headInfo} />
			</DragPreview>
		),
		canDrag,
	});

	const operationModeSource = operationMode?.source ?? null;
	const isActiveOperationModeSource =
		operationModeSource && operationSourceEquals(operationModeSource, source);

	const isActive = isDragging || isActiveOperationModeSource;

	return useRender({
		render,
		ref: dragRef,
		props: mergeProps<"div">(props, {
			className: classes(isActive && styles.activeSource),
		}),
	});
};
