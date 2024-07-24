import { type LogFn } from '@aztec/foundation/log';

import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

export function createL1Account(log: LogFn) {
  const privateKey = generatePrivateKey();
  const account = privateKeyToAccount(privateKey);

  log(`Private Key: ${privateKey}`);
  log(`Address: ${account.address}`);
}
