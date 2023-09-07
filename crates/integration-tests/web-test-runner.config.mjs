import { fileURLToPath } from 'url';
import { esbuildPlugin } from "@web/dev-server-esbuild";
import { playwrightLauncher } from "@web/test-runner-playwright";

export default {
  browsers: [
    playwrightLauncher({ product: "chromium" }),
    // playwrightLauncher({ product: "webkit" }),
    // playwrightLauncher({ product: "firefox" }),
  ],
  plugins: [
    esbuildPlugin({
      ts: true,
    }),
  ],
  files: ["test/integration/browser/**/*.test.ts"],
  nodeResolve: { browser: true },
  testFramework: {
    config: {
      ui: "bdd",
    //   timeout: 420000,
    },
  },
  rootDir:  fileURLToPath(new URL('./..', import.meta.url)),

};
