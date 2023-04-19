import { BufferReader, Fr } from '@aztec/foundation';
import { serializeToBuffer } from '../utils/serialize.js';

/**
 * A class representing the "preimage" of a function tree leaf.
 * @see abis/function_leaf_preimage.hpp
 */
export class FunctionLeafPreimage {
  readonly FUNCTION_SELECTOR_LENGTH = 4;

  constructor(public functionSelector: Buffer, public isPrivate: boolean, public vkHash: Fr, public acirHash: Fr) {
    this.assertFunctionSelectorLength(functionSelector);
  }

  /**
   * Assert the function selector buffer length matches `FUNCTION_SELECTOR_LENGTH`
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
    return serializeToBuffer(this.functionSelector, this.isPrivate, this.vkHash, this.acirHash);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionLeafPreimage {
    const reader = BufferReader.asReader(buffer);
    return new FunctionLeafPreimage(reader.readBytes(4), reader.readBoolean(), reader.readFr(), reader.readFr());
  }
}
