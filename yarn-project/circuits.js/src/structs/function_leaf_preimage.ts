import { BufferReader, Fr } from '@aztec/foundation';
import { serializeToBuffer } from '../utils/serialize.js';

/**
 * A class representing the "preimage" of a function tree leaf.
 * @see abis/function_leaf_preimage.hpp
 */
export class FunctionLeafPreimage {
  constructor(public functionSelector: Buffer, public isPrivate: boolean, public vkHash: Fr, public acirHash: Fr) {
    if (functionSelector.byteLength !== 4) {
      throw new Error(`Function selector must be 4 bytes long, got ${functionSelector.byteLength} bytes.`);
    }
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
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
