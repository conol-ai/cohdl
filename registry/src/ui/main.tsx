import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { RouteErrorPanel, routeTree } from "./routes";
import { parseCatalogueUrlSearch, stringifyCatalogueSearch } from "./catalogue-model";
import "./styles.css";

const queryClient = new QueryClient();
const router = createRouter({
  routeTree,
  defaultErrorComponent: RouteErrorPanel,
  // Repeatable class filters are part of the public `/search` URL contract.
  stringifySearch: (search) => stringifyCatalogueSearch(search),
  parseSearch: parseCatalogueUrlSearch,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
);
