import { createAztecRpcClient, waitForSandbox } from '@aztec/aztec.js';
import { aztecRpcTestSuite } from '@aztec/pxe';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

const setup = async () => {
  const aztecRpc = createAztecRpcClient(SANDBOX_URL);
  await waitForSandbox(aztecRpc);
  return aztecRpc;
};

aztecRpcTestSuite('aztec_rpc_sandbox', setup);
