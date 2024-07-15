import { type DebugLogger } from '@aztec/foundation/log';
import { createTXERpcServer } from '@aztec/txe';

import http from 'http';

const { TXE_PORT = 8081 } = process.env;

export const startTXE = (options: any, debugLogger: DebugLogger) => {
  debugLogger.info(`Setting up TXE...`);
  const txeServer = createTXERpcServer(debugLogger);
  const app = txeServer.getApp();
  const httpServer = http.createServer(app.callback());
  httpServer.timeout = 1e3 * 60 * 5; // 5 minutes
  const port = parseInt(options.txePort || TXE_PORT, 10);
  httpServer.listen(port);
  debugLogger.info(`TXE listening on port ${port}`);
};
