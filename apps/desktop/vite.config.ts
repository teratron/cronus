import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The desktop frontend. Tauri drives it at a fixed dev port and embeds the
// built `dist/` output. Tailwind is applied here, where the final CSS is bundled.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true },
});
