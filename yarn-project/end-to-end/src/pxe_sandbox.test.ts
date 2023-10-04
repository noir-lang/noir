import { createPXEClient, waitForSandbox } from '@aztec/aztec.js';
import { pxeTestSuite } from '@aztec/pxe';

const { PXE_URL = 'http://localhost:8080' } = process.env;

const setup = async () => {
  const pxe = createPXEClient(PXE_URL);
  await waitForSandbox(pxe);
  return pxe;
};

pxeTestSuite('pxe_sandbox', setup);
