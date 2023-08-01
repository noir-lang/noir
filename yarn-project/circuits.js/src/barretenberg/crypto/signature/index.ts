import { Fr } from '@aztec/foundation/fields';

import { PrivateKey } from '../../../types/private_key.js';

/**
 * Interface to represent a signature.
 */
export interface Signature {
  /**
   * Serializes to a buffer.
   * @returns A buffer.
   */
  toBuffer(): Buffer;
  /**
   * Serializes to an array of fields.
   * @returns Fields.
   */
  toFields(): Fr[];
}

/**
 * Interface to represent a signer.
 */
export interface Signer {
  /**
   * Signs the given message with the given private key.
   * @param message - What to sign.
   * @param privateKey - The private key.
   * @returns A signature.
   */
  constructSignature(message: Uint8Array, privateKey: PrivateKey): Signature;
}
