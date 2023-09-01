import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { startHttpRpcServer } from '@aztec/aztec-sandbox/http';
import { createDebugLogger } from '@aztec/aztec.js';
import { cliTestSuite } from '@aztec/cli';
import { AztecRPC } from '@aztec/types';

import { setup } from './fixtures/utils.js';

const HTTP_PORT = 9009;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let aztecNode: AztecNodeService | undefined;
let aztecRpcServer: AztecRPC;

const testSetup = async () => {
  const context = await setup(2);
  debug(`Environment set up`);
  const { deployL1ContractsValues } = context;
  ({ aztecNode, aztecRpcServer } = context);
  http = startHttpRpcServer(aztecRpcServer, deployL1ContractsValues, HTTP_PORT);
  debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  return aztecRpcServer;
};

const cleanup = async () => {
  http.close();
  await aztecNode?.stop();
  await (aztecRpcServer as AztecRPCServer).stop();
};

cliTestSuite('CLI e2e test', testSetup, cleanup, `http://localhost:${HTTP_PORT}`, debug);
