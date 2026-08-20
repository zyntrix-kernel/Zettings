import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Zettings } from "./app/zettings.tsx";
import { PERF_MARKS, perfMark } from "./lib/perf.ts";
import "./styles/zdl.css";

// Phase 9 cold-start anchor: the earliest script-eval timestamp available.
perfMark(PERF_MARKS.scriptStart);

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1 } },
});

const root = document.getElementById("root");
if (root === null) {
  throw new Error("Zettings: #root element missing in index.html");
}

perfMark(PERF_MARKS.reactMount);
createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Zettings />
    </QueryClientProvider>
  </StrictMode>,
);

// Phase 9 first-paint proxy: the earliest rAF after React commits.
requestAnimationFrame(() => perfMark(PERF_MARKS.firstPaint));
