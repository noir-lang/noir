import { DebugLogger } from '@aztec/foundation/log';
import { retryUntil } from '@aztec/foundation/retry';
import { PXE } from '@aztec/types';

export const waitForPXE = async (pxe: PXE, logger?: DebugLogger) => {
  await retryUntil(async () => {
    try {
      logger?.('Attempting to contact PXE...');
      await pxe.getNodeInfo();
      return true;
    } catch (error) {
      logger?.('Failed to contact PXE!');
    }
    return undefined;
  }, 'RPC Get Node Info');
};
