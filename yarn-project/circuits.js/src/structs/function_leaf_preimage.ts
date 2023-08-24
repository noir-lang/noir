import { FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../utils/serialize.js';

/**
 * A class representing the "preimage" of a function tree leaf.
 * @see abis/function_leaf_preimage.hpp
 */
export class FunctionLeafPreimage {
  constructor(
    /**
     * Function selector.
     */
    public functionSelector: FunctionSelector,
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
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
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
      reader.readObject(FunctionSelector),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readFr(),
      reader.readFr(),
    );
  }
}
