import { AztecNodeService } from '@aztec/aztec-node';
import { startHttpRpcServer } from '@aztec/aztec-sandbox';
import { PXE, createDebugLogger } from '@aztec/aztec.js';
import { PXEService } from '@aztec/pxe';

import { cliTestSuite } from './canary/cli.js';
import { setup as e2eSetup } from './fixtures/utils.js';

const HTTP_PORT = 9009;
const RPC_URL = `http://localhost:${HTTP_PORT}`;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let aztecNode: AztecNodeService | undefined;
let pxe: PXE;

const testSetup = async () => {
  const context = await e2eSetup(2);
  debug(`Environment set up`);
  ({ aztecNode, pxe } = context);
  http = startHttpRpcServer(pxe, HTTP_PORT);
  debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  return pxe;
};

const testCleanup = async () => {
  http.close();
  await aztecNode?.stop();
  await (pxe as PXEService).stop();
};

cliTestSuite('E2E CLI Test', testSetup, testCleanup, createDebugLogger('aztec:e2e_cli'), RPC_URL);
