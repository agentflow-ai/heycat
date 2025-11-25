import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
  await viteConfig(),
  defineConfig({
    test: {
      environment: "jsdom",
      globals: true,
      coverage: {
        provider: "v8",
        thresholds: {
          lines: 100,
          functions: 100,
        },
        exclude: [
          "**/*.test.ts",
          "**/*.test.tsx",
          "**/*.spec.ts",
          "**/*.spec.tsx",
          "**/node_modules/**",
          "**/dist/**",
          "**/*.config.ts",
        ],
      },
    },
  })
);
