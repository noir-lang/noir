import { Fr } from '@aztec/foundation/fields';

/**
 * Interface to represent a signature.
 */
export interface Signature {
  toBuffer(): Buffer;
  toFields(): Fr[];
}

/**
 * Interface to represent a signer.
 */
export interface Signer {
  constructSignature(message: Uint8Array, privateKey: Uint8Array): Signature;
}
