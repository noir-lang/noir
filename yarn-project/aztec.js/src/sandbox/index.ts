import { sleep } from '@aztec/foundation/sleep';
import { PXE } from '@aztec/types';

import { createPXEClient } from '../pxe_client.js';

export const { PXE_URL = 'http://localhost:8080' } = process.env;

/**
 * Function to wait until the sandbox becomes ready for use.
 * @param pxe - The pxe client connected to the sandbox.
 */
export async function waitForSandbox(pxe?: PXE) {
  pxe = pxe ?? createPXEClient(PXE_URL);
  while (true) {
    try {
      await pxe.getNodeInfo();
      break;
    } catch (err) {
      await sleep(1000);
    }
  }
}
