#!/usr/bin/env -S node --no-warnings
import { createAztecNodeClient } from '@aztec/circuit-types';
import { init } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';

import { getPXEServiceConfig } from '../config/index.js';
import { startPXEHttpServer } from '../pxe_http/index.js';
import { createPXEService } from '../pxe_service/index.js';

const { PXE_PORT = 8080, AZTEC_NODE_URL = 'http://localhost:8079' } = process.env;

const logger = createDebugLogger('aztec:pxe_service');

/**
 * Create and start a new PXE HTTP Server
 */
async function main() {
  logger.info(`Setting up PXE...`);

  await init();

  const pxeConfig = getPXEServiceConfig();
  const nodeRpcClient = createAztecNodeClient(AZTEC_NODE_URL);
  const pxeService = await createPXEService(nodeRpcClient, pxeConfig);

  const shutdown = async () => {
    logger.info('Shutting down...');
    await pxeService.stop();
    process.exit(0);
  };

  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);

  startPXEHttpServer(pxeService, PXE_PORT);
  logger.info(`PXE listening on port ${PXE_PORT}`);
}

main().catch(err => {
  logger.error(err);
  process.exit(1);
});
