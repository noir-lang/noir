import { Fr } from '@aztec/aztec.js';
import { deriveSigningKey } from '@aztec/circuits.js';

export function generateKeys() {
  const privateKey = Fr.random();
  const signingKey = deriveSigningKey(privateKey);
  return {
    privateEncryptionKey: privateKey,
    privateSigningKey: signingKey,
  };
}
