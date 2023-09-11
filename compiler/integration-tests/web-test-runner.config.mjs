import { fileURLToPath } from "url";
import { esbuildPlugin } from "@web/dev-server-esbuild";
import { webdriverLauncher } from "@web/test-runner-webdriver";

export default {
  browsers: [
    webdriverLauncher({
      automationProtocol: "webdriver",
      capabilities: {
        browserName: "firefox",
        "moz:firefoxOptions": {
          args: ["-headless"],
        },
      },
    }),
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
    },
  },
  rootDir: fileURLToPath(new URL("./../..", import.meta.url)),
  testsFinishTimeout: 60 * 10e3, // 10 minutes
};
