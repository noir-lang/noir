import { startHttpRpcServer } from '@aztec/aztec-sandbox';
import { PXE, createDebugLogger } from '@aztec/aztec.js';
import { createPXERpcServer } from '@aztec/pxe';

import { setup as e2eSetup } from './fixtures/utils.js';
import { cliTestSuite } from './shared/cli.js';

const HTTP_PORT = 9009;
let RPC_URL = `http://localhost:${HTTP_PORT}`;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let pxe: PXE;
let teardown: () => Promise<void>;

// Use Sandbox PXE URL if we're running against sandbox
const { PXE_URL } = process.env;
if (PXE_URL) {
  RPC_URL = PXE_URL;
}

const testSetup = async () => {
  const context = await e2eSetup(2);
  debug(`Environment set up`);
  ({ pxe, teardown } = context);
  if (!PXE_URL) {
    http = startHttpRpcServer(pxe, createPXERpcServer, HTTP_PORT);
    debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  }
  return pxe;
};

const testCleanup = async () => {
  http?.close();
  await teardown();
};

cliTestSuite('E2E CLI Test', testSetup, testCleanup, createDebugLogger('aztec:e2e_cli'), RPC_URL);
