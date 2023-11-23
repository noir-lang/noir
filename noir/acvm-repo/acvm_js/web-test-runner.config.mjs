import { esbuildPlugin } from "@web/dev-server-esbuild";
import { playwrightLauncher } from "@web/test-runner-playwright";

export default {
  browsers: [
    playwrightLauncher({ product: "chromium" }),
    playwrightLauncher({ product: "webkit" }),
    // Firefox requires 40s to perform a Pedersen hash so we recommend using either
    // a Chromium- or Webkit-based browser
    // playwrightLauncher({ product: "firefox" }),
  ],
  plugins: [
    esbuildPlugin({
      ts: true,
    }),
  ],
  files: ["test/browser/**/*.test.ts"],
  nodeResolve: true,
  testFramework: {
    config: {
      ui: "bdd",
      timeout: 40000,
    },
  },
};
