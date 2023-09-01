import { createAztecRpcClient, createDebugLogger, makeFetch } from '@aztec/aztec.js';
import { cliTestSuite } from '@aztec/cli';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

const debug = createDebugLogger('aztec:e2e_cli');

const setupRPC = () => {
  const aztecRpcClient = createAztecRpcClient(SANDBOX_URL, makeFetch([1, 2, 3], true));
  return Promise.resolve(aztecRpcClient);
};

cliTestSuite('CLI canary', setupRPC, () => Promise.resolve(), SANDBOX_URL, debug);
