import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

/**
 *
 */
export async function getAccounts(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const accounts = await client.getRegisteredAccounts();
  if (!accounts.length) {
    log('No accounts found.');
  } else {
    log(`Accounts found: \n`);
    for (const account of accounts) {
      log(account.toReadableString());
    }
  }
}
