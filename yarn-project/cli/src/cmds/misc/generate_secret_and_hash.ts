import { computeSecretHash } from '@aztec/aztec.js';
import { Fr } from '@aztec/foundation/fields';
import { type LogFn } from '@aztec/foundation/log';

export function generateSecretAndHash(log: LogFn) {
  const secret = Fr.random();

  // We hash this the same way that aztec nr hash does.
  const secretHash = computeSecretHash(secret);

  log(`
    Secret: ${secret}
    Secret hash: ${secretHash}
  `);
}
