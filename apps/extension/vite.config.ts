import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@solanostickers/core": path.resolve(__dirname, "../../packages/core/src/index.ts"),
      "@solanostickers/providers": path.resolve(__dirname, "../../packages/providers/src/index.ts"),
      "@solanostickers/media-web": path.resolve(__dirname, "../../packages/media-web/src/index.ts"),
      "@solanostickers/ui": path.resolve(__dirname, "../../packages/ui/src/index.ts")
    }
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        editor: path.resolve(__dirname, "editor.html"),
        background: path.resolve(__dirname, "src/background.ts")
      },
      output: {
        entryFileNames: "[name].js"
      }
    }
  }
});
