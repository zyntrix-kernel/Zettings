import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@zettings/bindings": fileURLToPath(
        new URL("../../packages/ts-bindings/src/index.ts", import.meta.url),
      ),
    },
  },
  // Tauri expects a fixed port; failing fast beats silent proxying.
  server: {
    port: 1420,
    strictPort: true,
  },
  clearScreen: false,
});
