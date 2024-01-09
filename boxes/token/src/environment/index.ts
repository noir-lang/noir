import { createPXEClient, waitForPXE } from '@aztec/aztec.js';

const { PXE_URL = 'http://localhost:8080' } = process.env;

// assumes environment is running locally, which this script does not trigger
// as well as anvil.  anvil can be started with yarn test:integration
export const setupEnvironment = async () => {
    const pxe = createPXEClient(PXE_URL);
    await waitForPXE(pxe);
    return pxe;
  };