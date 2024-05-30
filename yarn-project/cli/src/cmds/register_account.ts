import { type Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function registerAccount(
  rpcUrl: string,
  privateKey: Fr,
  partialAddress: Fr,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  const { address, publicKeys } = await client.registerAccount(privateKey, partialAddress);

  log(`\nRegistered account:\n`);
  log(`Address:         ${address.toString()}`);
  log(`Public key:      ${publicKeys.toString()}`);
  log(`Partial address: ${partialAddress.toString()}`);
}
