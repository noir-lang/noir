import { Fr } from '@aztec/aztec.js';

export function generateSecretKey() {
  return { secretKey: Fr.random() };
}
