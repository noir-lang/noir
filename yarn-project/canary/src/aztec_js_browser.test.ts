import { createDebugLogger, fileURLToPath } from '@aztec/aztec.js';
import { browserTestSuite } from '@aztec/end-to-end';

import Koa from 'koa';
import serve from 'koa-static';
import path, { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PORT = 3000;

const logger = createDebugLogger('aztec:canary_aztec.js:web');
const pageLogger = createDebugLogger('aztec:canary_aztec.js:web:page');

const setupApp = () => {
  const app = new Koa();
  app.use(serve(path.resolve(__dirname, './web')));
  const server = app.listen(PORT, () => {
    logger(`Server started at http://localhost:${PORT}`);
  });

  return server;
};

browserTestSuite(setupApp, pageLogger);
