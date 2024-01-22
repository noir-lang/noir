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
function _deriveSecretKey(secretKey: GrumpkinPrivateKey, index: Fr): GrumpkinPrivateKey {
  // TODO: Temporary hack. Should replace it with a secure way to derive the secret key.
  const hash = pedersenHash([secretKey.high, secretKey.low, index].map(v => v.toBuffer()));
  return new GrumpkinScalar(hash);
}

/**
 * Computes the nullifier secret key from seed secret key.
 */
export function computeNullifierSecretKey(seedSecretKey: GrumpkinPrivateKey): GrumpkinPrivateKey {
  // TODO
  // return deriveSecretKey(seedSecretKey, new Fr(1));
  return seedSecretKey;
}

/**
 * Computes the nullifier secret key for a contract.
 */
export function computeSiloedNullifierSecretKey(
  nullifierSecretKey: GrumpkinPrivateKey,
  _contractAddress: AztecAddress,
): GrumpkinPrivateKey {
  // TODO
  // return deriveSecretKey(nullifierSecretKey, contractAddress);
  return nullifierSecretKey;
}
