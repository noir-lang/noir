import { defaultReporter } from "@web/test-runner";
import { summaryReporter } from "@web/test-runner";
import { fileURLToPath } from "url";
import { esbuildPlugin } from "@web/dev-server-esbuild";
import { webdriverLauncher } from "@web/test-runner-webdriver";

let reporter = summaryReporter();
const debugPlugins = [];
// eslint-disable-next-line no-undef
if (process.env.CI !== "true" || process.env.RUNNER_DEBUG === "1") {
  reporter = defaultReporter();
  debugPlugins.push({
    name: "environment",
    serve(context) {
      if (context.path === "/compiler/integration-tests/test/environment.js") {
        return "export const TEST_LOG_LEVEL = 2;";
      }
    },
  });
}

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
    ...debugPlugins,
  ],
  files: ["test/integration/browser/recursion.test.ts"],
  nodeResolve: { browser: true },
  testFramework: {
    config: {
      ui: "bdd",
    },
  },
  // eslint-disable-next-line no-undef
  rootDir: fileURLToPath(new URL("./../..", import.meta.url)),
  testsFinishTimeout: 60 * 20e3, // 20 minutes
  reporters: [reporter],
};
