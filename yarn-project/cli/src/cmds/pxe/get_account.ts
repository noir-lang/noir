import { type AztecAddress } from '@aztec/aztec.js';
import { createCompatibleClient } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

export async function getAccount(aztecAddress: AztecAddress, rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const account = await client.getRegisteredAccount(aztecAddress);

  if (!account) {
    log(`Unknown account ${aztecAddress.toString()}`);
  } else {
    log(account.toReadableString());
  }
}
