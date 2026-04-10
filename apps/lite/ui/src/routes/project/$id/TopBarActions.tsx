import { createContext, type FC, type ReactNode, use } from "react";
import { createPortal } from "react-dom";

export const TopBarActionsPortalContext = createContext<HTMLElement | null>(null);

export const PositionedTopBarActions: FC<{
	children: ReactNode;
}> = ({ children }) => {
	const element = use(TopBarActionsPortalContext);
	if (!element) return null;

	return createPortal(children, element);
};
