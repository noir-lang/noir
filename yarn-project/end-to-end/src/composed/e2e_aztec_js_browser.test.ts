import { createDebugLogger, fileURLToPath } from '@aztec/aztec.js';
import { startPXEHttpServer } from '@aztec/pxe';

import Koa from 'koa';
import serve from 'koa-static';
import path, { dirname } from 'path';

import { setup } from '../fixtures/utils.js';
import { browserTestSuite } from '../shared/browser.js';

const { PXE_URL = '' } = process.env;

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PORT = 4000;
const PXE_PORT = 4001;

const logger = createDebugLogger('aztec:e2e_aztec_browser.js:web');
const pageLogger = createDebugLogger('aztec:e2e_aztec_browser.js:web:page');

/**
 * This test is a bit of a special case as it's on a web browser and not only on anvil and node.js.
 * To run the test, do the following:
 *    1) Build the whole repository,
 *    2) go to `yarn-project/end-to-end` and build the web packed package with `yarn build:web`,
 *    3) start anvil: `anvil`,
 *    4) if you intend to use a remotely running environment then export the URL of your PXE e.g.
 *       `export PXE_URL='http://localhost:8080'`
 *    5) go to `yarn-project/end-to-end` and run the test: `yarn test aztec_js_browser`
 *    6) If you get dependency error run `apt install libnss3 libatk1.0-0t64 libatk-bridge2.0-0t64 libcups2t64 libxdamage1 libxkbcommon0 libpango-1.0-0 libcairo2 libasound2t64`.
 *
 * NOTE 1: If you see the logs spammed with unexpected logs there is probably a chrome process with a webpage
 *         unexpectedly running in the background. Kill it with `killall chrome`
 * NOTE 2: Don't forget to run `yarn build:web` once you make changes!
 * NOTE 3: The test serializes token contract artifact to and from buffer. If you introduce a new type in the artifact
 *         you have to register it on `TypeRegistry` class, implement fromJSON method just like TypeRegistry requires
 *         and add a case in `contractArtifactFromBuffer(...)` function.
 */

const setupApp = async () => {
  const { pxe: pxeService } = await setup(0);
  let pxeURL = PXE_URL;
  let pxeServer = undefined;
  if (!PXE_URL) {
    pxeServer = startPXEHttpServer(pxeService, PXE_PORT);
    pxeURL = `http://localhost:${PXE_PORT}`;
  }

  const app = new Koa();
  app.use(serve(path.resolve(__dirname, '../web')));
  const server = app.listen(PORT, () => {
    logger.info(`Web Server started at http://localhost:${PORT}`);
  });

  return { server, webServerURL: `http://localhost:${PORT}`, pxeServer, pxeURL };
};

browserTestSuite(setupApp, pageLogger);
