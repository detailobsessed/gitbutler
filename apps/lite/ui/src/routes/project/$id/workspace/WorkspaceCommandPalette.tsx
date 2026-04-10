import { assert } from "#ui/routes/project/$id/shared.tsx";
import { formatShortcutKeys, type ShortcutBinding } from "#ui/shortcuts.ts";
import { Autocomplete, Dialog, ScrollArea } from "@base-ui/react";
import { type FC, use } from "react";
import type { WorkspaceCommand } from "./WorkspaceCommands.ts";
import { WorkspaceCommandRuntimeContext } from "./WorkspaceCommandRuntime.tsx";
import styles from "./WorkspaceCommandPalette.module.css";

const isPaletteCommandBinding = (binding: ShortcutBinding<WorkspaceCommand>): boolean =>
	binding.repeat === false;

interface Item {
	value: string;
	label: string;
	itemType: string;
	action: WorkspaceCommand;
}

interface Group {
	value: string;
	items: Array<Item>;
}

export const WorkspaceCommandPalette: FC<{
	open: boolean;
	onOpenChange: (open: boolean) => void;
}> = ({ open, onOpenChange }) => {
	const { runCommand, scope } = assert(use(WorkspaceCommandRuntimeContext));
	const groupedItems: Array<Group> = scope
		? [
				{
					value: scope.label,
					items: scope.bindings.filter(isPaletteCommandBinding).map((binding) => ({
						action: binding.action,
						itemType: formatShortcutKeys(binding.keys),
						label: binding.description,
						value: binding.id,
					})),
				},
			]
		: [];

	function handleItemClick(item: Item) {
		onOpenChange(false);
		runCommand(item.action);
	}

	return (
		<Dialog.Root open={open} onOpenChange={onOpenChange}>
			<Dialog.Portal>
				<Dialog.Backdrop className={styles.Backdrop} />
				<Dialog.Viewport className={styles.Viewport}>
					<Dialog.Popup aria-label="Command palette" className={styles.Popup}>
						<Autocomplete.Root
							open
							inline
							items={groupedItems}
							autoHighlight="always"
							keepHighlight
							itemToStringValue={(item) => item.label}
						>
							<Autocomplete.Input
								className={styles.Input}
								placeholder="Search for apps and commands..."
							/>
							<Dialog.Close className={styles.VisuallyHiddenClose}>
								Close command palette
							</Dialog.Close>

							<ScrollArea.Root className={styles.ListArea}>
								<ScrollArea.Viewport className={styles.ListViewport}>
									<ScrollArea.Content className={styles.ListContent}>
										<Autocomplete.Empty className={styles.Empty}>
											No results found.
										</Autocomplete.Empty>

										<Autocomplete.List className={styles.List}>
											{(group: Group) => (
												<Autocomplete.Group
													key={group.value}
													items={group.items}
													className={styles.Group}
												>
													<Autocomplete.GroupLabel className={styles.GroupLabel}>
														{group.value}
													</Autocomplete.GroupLabel>
													<Autocomplete.Collection>
														{(item: Item) => (
															<Autocomplete.Item
																key={item.value}
																value={item}
																className={styles.Item}
																onClick={() => handleItemClick(item)}
															>
																<span className={styles.ItemLabel}>{item.label}</span>
																<span className={styles.ItemType}>{item.itemType}</span>
															</Autocomplete.Item>
														)}
													</Autocomplete.Collection>
												</Autocomplete.Group>
											)}
										</Autocomplete.List>
									</ScrollArea.Content>
								</ScrollArea.Viewport>
								<ScrollArea.Scrollbar className={styles.Scrollbar}>
									<ScrollArea.Thumb className={styles.ScrollbarThumb} />
								</ScrollArea.Scrollbar>
							</ScrollArea.Root>
							<div className={styles.Footer}>
								<div className={styles.FooterLeft}>
									<span>Activate</span>
									<kbd className={styles.Kbd}>Enter</kbd>
								</div>
								<div className={styles.FooterRight}>
									<span>Actions</span>
									<kbd className={styles.Kbd}>Cmd</kbd>
									<kbd className={styles.Kbd}>K</kbd>
								</div>
							</div>
						</Autocomplete.Root>
					</Dialog.Popup>
				</Dialog.Viewport>
			</Dialog.Portal>
		</Dialog.Root>
	);
};
