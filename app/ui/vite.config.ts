import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Porta fixa: o Tauri carrega a UI de devUrl (app/src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "es2022", sourcemap: true, chunkSizeWarningLimit: 2000 },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["tests/**/*.test.ts", "tests/**/*.test.tsx"],
  },
});
