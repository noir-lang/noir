import { fileURLToPath } from 'url';
import { esbuildPlugin } from "@web/dev-server-esbuild";
// import { playwrightLauncher } from "@web/test-runner-playwright";
import { webdriverLauncher } from '@web/test-runner-webdriver';

export default {
  browsers: [
    // playwrightLauncher({ product: "chromium" }),
    // playwrightLauncher({ product: "webkit" }),
    // playwrightLauncher({ product: "firefox" }),
    webdriverLauncher({
      automationProtocol: 'webdriver',
      capabilities: {
        browserName: 'firefox',
        'moz:firefoxOptions': {
          args: ['-headless'],
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
    //   timeout: 420000,
    },
  },
  rootDir:  fileURLToPath(new URL('./../..', import.meta.url)),

};
