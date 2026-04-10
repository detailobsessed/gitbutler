import { formatShortcutKeys, ShortcutBinding } from "#ui/shortcuts.ts";
import { createContext, type FC, use } from "react";
import { createPortal } from "react-dom";
import styles from "./ShortcutsBar.module.css";

export const ShortcutsBarPortalContext = createContext<HTMLElement | null>(null);

const ShortcutsBar: FC<{
	label: string;
	items: Array<ShortcutBinding<unknown>>;
}> = ({ label, items }) => {
	const visibleItems = items.filter((item) => item.showInShortcutsBar !== false);

	return (
		<div className={styles.container}>
			<span className={styles.scope}>{label}</span>
			{visibleItems.map((item) => (
				<div key={item.id} className={styles.item}>
					<span className={styles.keys}>{formatShortcutKeys(item.keys)}</span>
					<span>{item.description}</span>
				</div>
			))}
		</div>
	);
};

export const PositionedShortcutsBar: FC<{
	label: string;
	items: Array<ShortcutBinding<unknown>>;
}> = ({ items, label }) => {
	const element = use(ShortcutsBarPortalContext);
	if (!element) return null;

	return createPortal(<ShortcutsBar label={label} items={items} />, element);
};
