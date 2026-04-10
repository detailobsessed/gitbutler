import { type Operation, useRunOperation } from "#ui/Operation.ts";
import { monitorForElements } from "@atlaskit/pragmatic-drag-and-drop/element/adapter";
import { FC, type ReactNode, useEffect } from "react";
import sharedStyles from "../-shared.module.css";
import { type OperationSource } from "./-OperationSource.ts";

export type DragData = {
	operationSource: OperationSource;
};

export const parseDragData = (data: unknown): OperationSource | null => {
	if (typeof data !== "object" || data === null || !("operationSource" in data)) return null;
	return (data as DragData).operationSource;
};

export type DropData = {
	operation: Operation | null;
	operationSource: OperationSource;
} | null;

const parseDropTargetData = (data: unknown): DropData | null => {
	if (typeof data !== "object" || data === null || !("operation" in data)) return null;
	return data as DropData;
};

export const DragPreview: FC<{ children: ReactNode }> = ({ children }) => (
	<div className={sharedStyles.dragPreview}>{children}</div>
);

export const useMonitorDraggedOperationSource = ({ projectId }: { projectId: string }) => {
	const runOperation = useRunOperation();

	useEffect(
		() =>
			monitorForElements({
				canMonitor: ({ source }) => parseDragData(source.data) !== null,
				onDrop: ({ location }) => {
					const dropData = location.current.dropTargets
						.map((dropTarget) => parseDropTargetData(dropTarget.data))
						.find((target) => target);

					if (!dropData?.operation) return;

					runOperation(projectId, dropData.operation);
				},
			}),
		[runOperation, projectId],
	);
};
