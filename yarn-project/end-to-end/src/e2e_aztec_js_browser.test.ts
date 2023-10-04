import { createDebugLogger, fileURLToPath } from '@aztec/aztec.js';

import Koa from 'koa';
import serve from 'koa-static';
import path, { dirname } from 'path';

import { browserTestSuite } from './canary/browser.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PORT = 3000;

const logger = createDebugLogger('aztec:canary_aztec.js:web');
const pageLogger = createDebugLogger('aztec:canary_aztec.js:web:page');
/**
 * This test is a bit of a special case as it's relying on sandbox and web browser and not only on anvil and node.js.
 * To run the test, do the following:
 *    1) Build the whole repository,
 *    2) go to `yarn-project/aztec.js` and build the web packed package with `yarn build:web`,
 *    3) start anvil: `anvil`,
 *    4) open new terminal and optionally set the more verbose debug level: `DEBUG=aztec:*`,
 *    5) go to the sandbox dir `yarn-project/aztec-sandbox` and run `yarn start`,
 *    6) open new terminal and export the URL of PXE from Sandbox: `export PXE_URL='http://localhost:8080'`,
 *    7) go to `yarn-project/end-to-end` and run the test: `yarn test aztec_js_browser`
 *
 * NOTE: If you see aztec-sandbox logs spammed with unexpected logs there is probably a chrome process with a webpage
 *       unexpectedly running in the background. Kill it with `killall chrome`
 */
const setupApp = () => {
  const app = new Koa();
  app.use(serve(path.resolve(__dirname, './web')));
  const server = app.listen(PORT, () => {
    logger(`Server started at http://localhost:${PORT}`);
  });

  return server;
};

browserTestSuite(setupApp, pageLogger);
