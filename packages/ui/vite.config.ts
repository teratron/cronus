import { resolve } from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Library build: `@cronus/ui` is consumed from source by the desktop app in dev;
// this build produces a distributable bundle. React stays external. Tailwind is
// applied by the consuming app's build (where the final CSS is bundled).
export default defineConfig({
  plugins: [
    react(),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, "src/index.ts"),
      formats: [
        "es",
      ],
      fileName: "index",
    },
    rollupOptions: {
      external: [
        "react",
        "react-dom",
        "react/jsx-runtime",
      ],
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: [
      "./vitest.setup.ts",
    ],
  },
});
