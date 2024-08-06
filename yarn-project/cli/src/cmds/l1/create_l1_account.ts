import { type LogFn } from '@aztec/foundation/log';

import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

import { prettyPrintJSON } from '../../utils/commands.js';

export function createL1Account(json: boolean, log: LogFn) {
  const privateKey = generatePrivateKey();
  const account = privateKeyToAccount(privateKey);

  if (json) {
    log(prettyPrintJSON({ privateKey, address: account.address }));
  } else {
    log(`Private Key: ${privateKey}`);
    log(`Address: ${account.address}`);
  }
}
