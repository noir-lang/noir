import { type Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function registerAccount(
  rpcUrl: string,
  secretKey: Fr,
  partialAddress: Fr,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  const { address, publicKey } = await client.registerAccount(secretKey, partialAddress);

  log(`\nRegistered account:\n`);
  log(`Address:         ${address.toString()}`);
  log(`Public key:      ${publicKey.toString()}`);
  log(`Partial address: ${partialAddress.toString()}`);
}
