import { defaultReporter } from "@web/test-runner";
import { summaryReporter } from "@web/test-runner";
import { fileURLToPath } from "url";
import { esbuildPlugin } from "@web/dev-server-esbuild";
import { webdriverLauncher } from "@web/test-runner-webdriver";

// eslint-disable-next-line no-undef
const reporter = process.env.CI ? summaryReporter() : defaultReporter();

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
  // eslint-disable-next-line no-undef
  rootDir: fileURLToPath(new URL("./../..", import.meta.url)),
  testsFinishTimeout: 60 * 20e3, // 20 minutes
  reporters: [reporter],
};
