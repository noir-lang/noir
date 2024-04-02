import { type L1ContractAddresses } from '@aztec/ethereum';
import { retryUntil } from '@aztec/foundation/retry';

import { createPXEClient } from '../rpc_clients/index.js';

export const getL1ContractAddresses = async (url: string): Promise<L1ContractAddresses> => {
  const pxeClient = createPXEClient(url);
  const response = await retryUntil(
    async () => {
      try {
        return (await pxeClient.getNodeInfo()).l1ContractAddresses;
      } catch (err) {
        // do nothing
      }
    },
    'isNodeReady',
    120,
    1,
  );
  return response;
};
