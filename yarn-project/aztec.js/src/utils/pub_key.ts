import { PrivateKey, PublicKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

/**
 * Method for generating a public grumpkin key from a private key.
 * @param privateKey - The private key.
 * @returns The generated public key.
 */
export async function generatePublicKey(privateKey: PrivateKey): Promise<PublicKey> {
  const grumpkin = await Grumpkin.new();
  return grumpkin.mul(grumpkin.generator(), privateKey);
}
