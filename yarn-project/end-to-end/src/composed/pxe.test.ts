import { waitForPXE } from '@aztec/aztec.js';
import { pxeTestSuite } from '@aztec/pxe';

import { setup } from '../fixtures/utils.js';

const setupEnv = async () => {
  const { pxe } = await setup(0);
  await waitForPXE(pxe);
  return pxe;
};

pxeTestSuite('pxe', setupEnv);
