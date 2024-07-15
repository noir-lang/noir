#!/usr/bin/env -S node --no-warnings
import { createDebugLogger } from '@aztec/aztec.js';

import http from 'http';

import { createTXERpcServer } from '../index.js';

/**
 * Create and start a new TXE HTTP Server
 */
function main() {
  const { TXE_PORT = 8080 } = process.env;

  const logger = createDebugLogger('aztec:txe_service');
  logger.info(`Setting up TXE...`);

  const txeServer = createTXERpcServer(logger);
  const app = txeServer.getApp();
  const httpServer = http.createServer(app.callback());
  httpServer.timeout = 1e3 * 60 * 5; // 5 minutes
  httpServer.listen(TXE_PORT);

  logger.info(`TXE listening on port ${TXE_PORT}`);
}

main();
