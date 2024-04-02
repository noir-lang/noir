import { type Fr } from '@aztec/foundation/fields';

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
