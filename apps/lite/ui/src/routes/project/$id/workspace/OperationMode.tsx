import { type Operation } from "#ui/Operation.ts";
import { FileParent } from "#ui/domain/FileParent.ts";
import { Match } from "effect";
import { type Item } from "./Item.ts";
import {
	getBranchTargetOperation,
	getCombineOperation,
	getCommitTargetMoveOperation,
	getTearOffBranchTargetOperation,
	type ResolvedOperationSource,
} from "./ResolvedOperationSource.ts";
import { operationSourceMatchesItem } from "./OperationSource.ts";
import { type OperationMode } from "./WorkspaceMode.ts";

const rubModeOperationSourceToOperation = ({
	resolvedOperationSource,
	target,
}: {
	resolvedOperationSource: ResolvedOperationSource;
	target: Item;
}) =>
	Match.value(target).pipe(
		Match.tags({
			ChangesSection: (target) =>
				getCombineOperation({
					resolvedOperationSource,
					target: FileParent.ChangesSection({ stackId: target.stackId }),
				}),
			Commit: (target) =>
				getCombineOperation({
					resolvedOperationSource,
					target: FileParent.Commit({ commitId: target.commitId }),
				}),
		}),
		Match.orElse(() => null),
	);

const moveModeOperationSourceToOperation = ({
	resolvedOperationSource,
	target,
}: {
	resolvedOperationSource: ResolvedOperationSource;
	target: Item;
}) =>
	Match.value(target).pipe(
		Match.tags({
			Segment: ({ branchRef }) =>
				branchRef === null
					? null
					: getBranchTargetOperation({
							resolvedOperationSource,
							branchRef,
						}),
			Commit: (target) =>
				getCommitTargetMoveOperation({
					resolvedOperationSource,
					commitId: target.commitId,
					side: "below",
				}),
			BaseCommit: () => getTearOffBranchTargetOperation(resolvedOperationSource),
		}),
		Match.orElse(() => null),
	);

export const operationModeToOperation = ({
	operationMode,
	resolvedOperationSource,
	target,
}: {
	operationMode: OperationMode;
	resolvedOperationSource: ResolvedOperationSource;
	target: Item;
}): Operation | null =>
	Match.value(operationMode).pipe(
		Match.tagsExhaustive({
			Rub: () => rubModeOperationSourceToOperation({ resolvedOperationSource, target }),
			Move: () => moveModeOperationSourceToOperation({ resolvedOperationSource, target }),
		}),
	);

export const isOperationModeSourceOrTarget = ({
	item,
	operationMode,
	resolvedOperationSource,
}: {
	item: Item;
	operationMode: OperationMode;
	resolvedOperationSource: ResolvedOperationSource | null;
}): boolean =>
	operationSourceMatchesItem(operationMode.source, item) ||
	(!!resolvedOperationSource &&
		!!operationModeToOperation({
			operationMode,
			resolvedOperationSource,
			target: item,
		}));
