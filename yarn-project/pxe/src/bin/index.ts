#!/usr/bin/env -S node --no-warnings
import { createDebugLogger } from '@aztec/foundation/log';
import { createAztecNodeRpcClient } from '@aztec/types';

import { getPXEServiceConfig } from '../config/index.js';
import { startPXEHttpServer } from '../pxe_http/index.js';
import { createPXEService } from '../pxe_service/index.js';

const { SERVER_PORT = 8080, AZTEC_NODE_RPC_URL = '' } = process.env;

const logger = createDebugLogger('aztec:pxe_service');

/**
 * Create and start a new PXE HTTP Server
 */
async function main() {
  logger.info(`Setting up PXE...`);

  const pxeConfig = getPXEServiceConfig();
  const nodeRpcClient = createAztecNodeRpcClient(AZTEC_NODE_RPC_URL);
  const pxeService = await createPXEService(nodeRpcClient, pxeConfig);

  const shutdown = async () => {
    logger.info('Shutting down...');
    await pxeService.stop();
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  startPXEHttpServer(pxeService, SERVER_PORT);
  logger.info(`PXE listening on port ${SERVER_PORT}`);
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
