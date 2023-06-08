import { CircuitsWasm, Fr } from '@aztec/circuits.js';
import { computeSecretMessageHash } from '@aztec/circuits.js/abis';

/**
 * Given a secret, it computes its pedersen hash - used to send l1 to l2 messages
 * @param secret - the secret to hash - secret could be generated however you want e.g. `Fr.random()`
 * @returns the hash
 */
export async function computeMessageSecretHash(secret: Fr): Promise<Fr> {
  const wasm = await CircuitsWasm.get();
  return computeSecretMessageHash(wasm, secret);
}
