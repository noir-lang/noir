import { defaultReporter } from '@web/test-runner';
import { summaryReporter } from '@web/test-runner';
import { fileURLToPath } from 'url';
import { esbuildPlugin } from '@web/dev-server-esbuild';
import { playwrightLauncher } from '@web/test-runner-playwright';
import { importMapsPlugin } from '@web/dev-server-import-maps';

let reporter = summaryReporter();
const debugPlugins = [];

if (process.env.CI !== 'true' || process.env.RUNNER_DEBUG === '1') {
  reporter = defaultReporter();
  debugPlugins.push({
    name: 'environment',
    serve(context) {
      if (context.path === '/compiler/integration-tests/test/environment.js') {
        return `export const TEST_LOG_LEVEL = 2;`;
      }
    },
  });
}

export default {
  browsers: [
    playwrightLauncher({
      product: 'chromium',
    }),
    // playwrightLauncher({ product: "webkit" }),
    // playwrightLauncher({ product: "firefox" }),
  ],
  concurrency: 1,
  concurrentBrowsers: 1,
  middleware: [
    async (ctx, next) => {
      if (ctx.url.endsWith('.wasm.gz')) {
        ctx.url = ctx.url.replace('/', '/node_modules/@aztec/bb.js/dest/browser/');
      }
      // Mock pino as its not supported on ESM environment
      // In our tests we are overriding the logger to tslog anyway
      if (ctx.url.includes('pino/browser.js')) {
        ctx.url = '/compiler/integration-tests/test/mocks/pino.js';
      }
      if (ctx.url.includes('buffer/index.js')) {
        ctx.url = '/compiler/integration-tests/test/mocks/buffer.js';
      }
      await next();
    },
  ],
  testRunnerHtml: (testFramework) =>
    // Polyfill Buffer
    `<!DOCTYPE html>
    <html>
      <body>
        <script>
          // Force bind fetch
          globalThis.fetch = globalThis.fetch.bind(globalThis);
        </script>
        <script type="module" src="/compiler/integration-tests/test/mocks/buffer.js"></script>
        <script type="module" src="${testFramework}"></script>
      </body>
    </html>`,
  plugins: [
    esbuildPlugin({
      ts: true,
    }),
    importMapsPlugin({
      inject: {
        importMap: {
          imports: {
            // mock os module
            os: '/test/mocks/os.js',
          },
        },
      },
    }),
    ...debugPlugins,
  ],
  files: ['test/browser/**/*.test.ts'],
  nodeResolve: { browser: true },
  testFramework: {
    config: {
      ui: 'bdd',
    },
  },

  rootDir: fileURLToPath(new URL('./../..', import.meta.url)),
  testsFinishTimeout: 60 * 20e3, // 20 minutes
  reporters: [reporter],
};
