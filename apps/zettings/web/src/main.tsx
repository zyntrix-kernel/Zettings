import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/zdl.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("Zettings shell bootstrap failed: missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
