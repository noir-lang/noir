import { type AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';

export async function getRecipient(aztecAddress: AztecAddress, rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const recipient = await client.getRecipient(aztecAddress);

  if (!recipient) {
    log(`Unknown recipient ${aztecAddress.toString()}`);
  } else {
    log(recipient.toReadableString());
  }
}
