import { AztecAddress, Fr, Point } from '@aztec/aztec.js';
import { CompleteAddress } from '@aztec/circuit-types';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function registerRecipient(
  aztecAddress: AztecAddress,
  publicKey: Point,
  partialAddress: Fr,
  rpcUrl: string,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  await client.registerRecipient(CompleteAddress.create(aztecAddress, publicKey, partialAddress));
  log(`\nRegistered details for account with address: ${aztecAddress}\n`);
}
