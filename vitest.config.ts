import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  resolve: {
    alias: {
      "@solanostickers/core": path.resolve(__dirname, "packages/core/src/index.ts"),
      "@solanostickers/providers": path.resolve(__dirname, "packages/providers/src/index.ts"),
      "@solanostickers/media-web": path.resolve(__dirname, "packages/media-web/src/index.ts"),
      "@solanostickers/ui": path.resolve(__dirname, "packages/ui/src/index.ts")
    }
  },
  test: {
    include: [
      "packages/**/test/**/*.test.ts",
      "packages/**/test/**/*.test.tsx",
      "apps/**/test/**/*.test.ts",
      "apps/**/test/**/*.test.tsx"
    ],
    environment: "node"
  }
});
