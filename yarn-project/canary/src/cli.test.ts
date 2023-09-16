import { createAztecRpcClient, createDebugLogger, makeFetch, waitForSandbox } from '@aztec/aztec.js';
import { cliTestSuite } from '@aztec/end-to-end';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

const debug = createDebugLogger('aztec:canary_cli');

const setupRPC = async () => {
  const aztecRpcClient = createAztecRpcClient(SANDBOX_URL, makeFetch([1, 2, 3, 4, 5], true));
  await waitForSandbox(aztecRpcClient);
  return aztecRpcClient;
};

cliTestSuite('CLI Canary', setupRPC, () => Promise.resolve(), debug, SANDBOX_URL);
