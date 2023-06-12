import { EcdsaSignature } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Point } from '@aztec/foundation/fields';

/** Represents a user public key. */
export type PublicKey = Point;

/**
 * Convert the current Point instance to an AztecAddress.
 * Takes the first 20 bytes of the point's buffer and creates an AztecAddress instance from it.
 *
 * @returns An AztecAddress instance representing the address corresponding to this point.
 * @deprecated To be removed once we go full account abstraction.
 */
export function getAddressFromPublicKey(pubKey: PublicKey) {
  return AztecAddress.fromBuffer(pubKey.buffer.slice(0, AztecAddress.SIZE_IN_BYTES));
}

/**
 * Represents a secure storage for managing keys.
 * Provides functionality to create and retrieve accounts, private and public keys,
 * signing public keys, as well as signing transaction requests using ECDSA signatures.
 */
export interface KeyStore {
  createAccount(): Promise<PublicKey>;
  addAccount(privKey: Buffer): Promise<PublicKey>;
  getAccounts(): Promise<PublicKey[]>;
  getAccountPrivateKey(pubKey: PublicKey): Promise<Buffer>;
  ecdsaSign(what: Buffer, from: PublicKey): Promise<EcdsaSignature>;
}
