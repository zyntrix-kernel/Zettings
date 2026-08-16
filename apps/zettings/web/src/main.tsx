import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Zettings } from "./app/zettings.tsx";
import "./styles/zdl.css";

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1 } },
});

const root = document.getElementById("root");
if (root === null) {
  throw new Error("Zettings: #root element missing in index.html");
}
createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Zettings />
    </QueryClientProvider>
  </StrictMode>,
);
