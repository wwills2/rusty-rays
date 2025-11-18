import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";

// https://vite.dev/config/
export default defineConfig({
  root: resolve(__dirname, "src", "renderer"),
  base: "", // important for Electron so paths are relative in prod
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src", "renderer"),
    },
  },
  build: {
    outDir: resolve(__dirname, "build", "renderer"),
    emptyOutDir: true,
  },
});
