import { type CompleteAddress, type PXE } from '@aztec/circuit-types';
import { retryUntil } from '@aztec/foundation/retry';

import { DefaultWaitOpts, type WaitOpts } from '../contract/index.js';

/**
 * Waits for the account to finish synchronizing with the PXE Service.
 * @param pxe - PXE instance
 * @param address - Address to wait for synch
 * @param opts - Wait options
 */
export async function waitForAccountSynch(
  pxe: PXE,
  address: CompleteAddress,
  { interval, timeout }: WaitOpts = DefaultWaitOpts,
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
