import { createDebugLogger, createPXEClient, makeFetch, waitForSandbox } from '@aztec/aztec.js';
import { cliTestSuite } from '@aztec/end-to-end';

const { PXE_URL = 'http://localhost:8080' } = process.env;

const debug = createDebugLogger('aztec:canary_cli');

const setupRPC = async () => {
  const pxe = createPXEClient(PXE_URL, makeFetch([1, 2, 3, 4, 5], true));
  await waitForSandbox(pxe);
  return pxe;
};

cliTestSuite('CLI Canary', setupRPC, () => Promise.resolve(), debug, PXE_URL);
