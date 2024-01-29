import { PXE, createDebugLogger } from '@aztec/aztec.js';
import { startHttpRpcServer } from '@aztec/foundation/json-rpc/server';
import { createPXERpcServer } from '@aztec/pxe';

import { setup as e2eSetup } from './fixtures/utils.js';
import { cliTestSuite } from './shared/cli.js';

const HTTP_PORT = 9009;
const debug = createDebugLogger('aztec:e2e_cli');

const { PXE_URL = '' } = process.env;
let RPC_URL = PXE_URL;

let http: ReturnType<typeof startHttpRpcServer>;
let pxe: PXE;
let teardown: () => Promise<void>;

const testSetup = async () => {
  const context = await e2eSetup(2);
  debug(`Environment set up`);
  ({ pxe, teardown } = context);
  if (!RPC_URL) {
    http = startHttpRpcServer('pxe', pxe, createPXERpcServer, HTTP_PORT);
    debug(`HTTP RPC server started on port ${HTTP_PORT}`);
    RPC_URL = `http://localhost:${HTTP_PORT}`;
  }
  return { pxe, rpcURL: RPC_URL };
};

const testCleanup = async () => {
  http?.close();
  await teardown();
};

cliTestSuite('E2E CLI Test', testSetup, testCleanup, createDebugLogger('aztec:e2e_cli'));
