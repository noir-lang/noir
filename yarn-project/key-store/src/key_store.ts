import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Point } from '@aztec/foundation/fields';
import { Curve, Signature, Signer } from '@aztec/circuits.js/barretenberg';

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
  return AztecAddress.fromBuffer(pubKey.toBuffer().subarray(0, AztecAddress.SIZE_IN_BYTES));
}

/**
 * Represents a secure storage for managing keys.
 * Provides functionality to create and retrieve accounts, private and public keys,
 * signing public keys, as well as signing transaction requests.
 */
export interface KeyStore {
  createAccount(curve: Curve, signer: Signer): Promise<PublicKey>;
  addAccount(curve: Curve, signer: Signer, privKey: Buffer): PublicKey;
  getAccounts(): Promise<PublicKey[]>;
  getAccountPrivateKey(pubKey: PublicKey): Promise<Buffer>;
  sign(what: Buffer, from: PublicKey): Promise<Signature>;
}
