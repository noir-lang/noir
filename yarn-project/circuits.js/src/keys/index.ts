import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';

import { Grumpkin } from '../barretenberg/crypto/grumpkin/index.js';
import { GrumpkinPrivateKey } from '../types/grumpkin_private_key.js';

/**
 *  Derives the public key of a secret key.
 */
export function derivePublicKey(secretKey: GrumpkinPrivateKey) {
  const grumpkin = new Grumpkin();
  return grumpkin.mul(grumpkin.generator(), secretKey);
}

/**
 * Derives a new secret key from a secret key and an index.
 */
function deriveSecretKey(secretKey: GrumpkinPrivateKey, index: Fr): GrumpkinPrivateKey {
  // TODO: Temporary hack. Should replace it with a secure way to derive the secret key.
  // Match the way keys are derived in noir-protocol-circuits/src/crates/private_kernel_lib/src/common.nr
  const hash = pedersenHash([secretKey.high, secretKey.low, index].map(v => v.toBuffer()));
  return new GrumpkinScalar(hash);
}

/**
 * Computes the nullifier secret key from seed secret key.
 */
export function computeNullifierSecretKey(seedSecretKey: GrumpkinPrivateKey): GrumpkinPrivateKey {
  return deriveSecretKey(seedSecretKey, new Fr(1));
}

/**
 * Computes the nullifier secret key for a contract.
 */
export function computeSiloedNullifierSecretKey(
  nullifierSecretKey: GrumpkinPrivateKey,
  contractAddress: AztecAddress,
): GrumpkinPrivateKey {
  return deriveSecretKey(nullifierSecretKey, contractAddress);
}
