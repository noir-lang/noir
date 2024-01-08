import { retryUntil } from '@aztec/foundation/retry';
import { CompleteAddress, PXE } from '@aztec/types';

import { WaitOpts } from '../contract/index.js';

/**
 * Waits for the account to finish synchronizing with the PXE Service.
 * @param pxe - PXE instance
 * @param address - Address to wait for synch
 * @param opts - Wait options
 */
export async function waitForAccountSynch(
  pxe: PXE,
  address: CompleteAddress,
  { interval, timeout }: WaitOpts,
): Promise<void> {
  const publicKey = address.publicKey.toString();
  await retryUntil(
    async () => {
      const status = await pxe.getSyncStatus();
      const accountSynchedToBlock = status.notes[publicKey];
      if (typeof accountSynchedToBlock === 'undefined') {
        return false;
      } else {
        return accountSynchedToBlock >= status.blocks;
      }
    },
    'waitForAccountSynch',
    timeout,
    interval,
  );
}
