import { startHttpRpcServer } from '@aztec/aztec-sandbox';
import { PXE, createDebugLogger } from '@aztec/aztec.js';
import { createPXERpcServer } from '@aztec/pxe';

import { cliTestSuite } from './canary/cli.js';
import { setup as e2eSetup } from './fixtures/utils.js';

const HTTP_PORT = 9009;
const RPC_URL = `http://localhost:${HTTP_PORT}`;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let pxe: PXE;
let teardown: () => Promise<void>;

const testSetup = async () => {
  const context = await e2eSetup(2);
  debug(`Environment set up`);
  ({ pxe, teardown } = context);
  http = startHttpRpcServer(pxe, createPXERpcServer, HTTP_PORT);
  debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  return pxe;
};

const testCleanup = async () => {
  http.close();
  await teardown();
};

cliTestSuite('E2E CLI Test', testSetup, testCleanup, createDebugLogger('aztec:e2e_cli'), RPC_URL);
