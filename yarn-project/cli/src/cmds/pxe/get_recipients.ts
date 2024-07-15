import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';

export async function getRecipients(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const recipients = await client.getRecipients();
  if (!recipients.length) {
    log('No recipients found.');
  } else {
    log(`Recipients found: \n`);
    for (const recipient of recipients) {
      log(recipient.toReadableString());
    }
  }
}
