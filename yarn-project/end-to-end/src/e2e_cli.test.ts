import { AztecNodeService } from '@aztec/aztec-node';
import { startHttpRpcServer } from '@aztec/aztec-sandbox';
import { AztecRPC, createDebugLogger } from '@aztec/aztec.js';
import { AztecRPCServer } from '@aztec/pxe';

import { cliTestSuite } from './canary/cli.js';
import { setup as e2eSetup } from './fixtures/utils.js';

const HTTP_PORT = 9009;
const RPC_URL = `http://localhost:${HTTP_PORT}`;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let aztecNode: AztecNodeService | undefined;
let aztecRpcServer: AztecRPC;

const testSetup = async () => {
  const context = await e2eSetup(2);
  debug(`Environment set up`);
  ({ aztecNode, aztecRpcServer } = context);
  http = startHttpRpcServer(aztecRpcServer, HTTP_PORT);
  debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  return aztecRpcServer;
};

const testCleanup = async () => {
  http.close();
  await aztecNode?.stop();
  await (aztecRpcServer as AztecRPCServer).stop();
};

cliTestSuite('E2E CLI Test', testSetup, testCleanup, createDebugLogger('aztec:e2e_cli'), RPC_URL);
