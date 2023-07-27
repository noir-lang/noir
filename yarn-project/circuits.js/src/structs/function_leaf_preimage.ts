import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../utils/serialize.js';

/**
 * A class representing the "preimage" of a function tree leaf.
 * @see abis/function_leaf_preimage.hpp
 */
export class FunctionLeafPreimage {
  readonly FUNCTION_SELECTOR_LENGTH = 4;

  constructor(
    /**
     * Function selector `FUNCTION_SELECTOR_LENGTH` bytes long.
     */
    public functionSelector: Buffer,
    /**
     * Indicates whether the function is only callable by self or not.
     */
    public isInternal: boolean,
    /**
     * Indicates whether the function is private or public.
     */
    public isPrivate: boolean,
    /**
     * Verification key hash of the function.
     */
    public vkHash: Fr,
    /**
     * Hash of the ACIR of the function.
     */
    public acirHash: Fr,
  ) {
    this.assertFunctionSelectorLength(functionSelector);
  }

  /**
   * Assert the function selector buffer length matches `FUNCTION_SELECTOR_LENGTH`.
   * @param functionSelector - The buffer containing the function selector.
   * @throws If the function selector buffer length does not match `FUNCTION_SELECTOR_LENGTH`.
   */
  private assertFunctionSelectorLength(functionSelector: Buffer) {
    if (functionSelector.byteLength !== this.FUNCTION_SELECTOR_LENGTH) {
      throw new Error(
        `Function selector must be ${this.FUNCTION_SELECTOR_LENGTH} bytes long, got ${functionSelector.byteLength} bytes.`,
      );
    }
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    this.assertFunctionSelectorLength(this.functionSelector);
    return serializeToBuffer(this.functionSelector, this.isInternal, this.isPrivate, this.vkHash, this.acirHash);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of FunctionLeafPreimage.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionLeafPreimage {
    const reader = BufferReader.asReader(buffer);
    return new FunctionLeafPreimage(
      reader.readBytes(4),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readFr(),
      reader.readFr(),
    );
  }
}
