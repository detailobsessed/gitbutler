import { Toast, ToastManager, Tooltip } from "@base-ui/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { RegisteredRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { Provider } from "react-redux";
import { store } from "#ui/state/store.ts";
import { Toasts } from "./Toasts";

export const App: React.FC<{
	queryClient: QueryClient;
	toastManager: ToastManager;
	router: RegisteredRouter;
}> = ({ queryClient, toastManager, router }) => (
	<StrictMode>
		<Provider store={store}>
			<QueryClientProvider client={queryClient}>
				<Toast.Provider toastManager={toastManager}>
					<Tooltip.Provider>
						<RouterProvider router={router} />
						<Toasts />
					</Tooltip.Provider>
				</Toast.Provider>
				<ReactQueryDevtools />
			</QueryClientProvider>
		</Provider>
	</StrictMode>
);
