import { AztecAddress, PartialAddress, PublicKey } from '@aztec/circuits.js';

/** Address and preimages associated with an account. */
export type CompleteAddress = {
  /** Address of an account. Derived from the partial address and public key. */
  address: AztecAddress;
  /** Partial address of the account. Required for deriving the address from the encryption public key. */
  partialAddress: PartialAddress;
  /** Encryption public key associated with this address. */
  publicKey: PublicKey;
};

/** Returns whether the argument looks like a CompleteAddress. */
export function isCompleteAddress(obj: any): obj is CompleteAddress {
  if (!obj) return false;
  const maybe = obj as CompleteAddress;
  return !!maybe.address && !!maybe.partialAddress && !!maybe.publicKey;
}
