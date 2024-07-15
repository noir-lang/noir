import { type AztecAddress, type Fr } from '@aztec/aztec.js';
import { CompleteAddress } from '@aztec/circuit-types';
import { type PublicKeys } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';

export async function registerRecipient(
  aztecAddress: AztecAddress,
  publicKeys: PublicKeys,
  partialAddress: Fr,
  rpcUrl: string,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  await client.registerRecipient(new CompleteAddress(aztecAddress, publicKeys, partialAddress));
  log(`\nRegistered details for account with address: ${aztecAddress}\n`);
}
