import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { createDebugLogger } from '@aztec/foundation/log';

import 'dotenv/config';
import http from 'http';

import { appFactory } from './app.js';

const logger = createDebugLogger('aztec:rollup_provider');

const { SERVER_PORT = 9000 } = process.env;

/**
 * Entrypoint for the rollup provider service
 * @returns An empty promise
 */
async function main() {
  logger('Server started...');
  const aztecNodeConfig: AztecNodeConfig = getConfigEnvVars();
  const node = await AztecNodeService.createAndSync(aztecNodeConfig);

  const shutdown = async () => {
    await node.stop();
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  const app = appFactory(node, '');

  const httpServer = http.createServer(app.callback());
  httpServer.listen(SERVER_PORT);
  logger(`Server listening on port ${SERVER_PORT}.`);
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
