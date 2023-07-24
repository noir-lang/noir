import { PublicKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

import { Point } from '../index.js';

/**
 * Method for generating a public grumpkin key from a private key.
 * @param privateKey - The private key.
 * @returns The generated public key.
 */
export async function generatePublicKey(privateKey: Buffer): Promise<PublicKey> {
  const grumpkin = await Grumpkin.new();
  return Point.fromBuffer(grumpkin.mul(grumpkin.generator(), privateKey));
}
