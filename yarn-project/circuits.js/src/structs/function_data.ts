import { BufferReader } from '@aztec/foundation/serialize';
import { serializeToBuffer } from '../utils/serialize.js';

/**
 * Function description for circuit.
 * @see abis/function_data.hpp
 */
export class FunctionData {
  constructor(public functionSelector: Buffer, public isPrivate = true, public isConstructor = true) {
    if (functionSelector.byteLength !== 4) {
      throw new Error(`Function selector must be 4 bytes long, got ${functionSelector.byteLength} bytes.`);
    }
  }
  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.functionSelector, this.isPrivate, this.isConstructor);
  }

  public static empty() {
    return new FunctionData(Buffer.alloc(4, 0));
  }
  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionData {
    const reader = BufferReader.asReader(buffer);
    return new FunctionData(reader.readBytes(4), reader.readBoolean(), reader.readBoolean());
  }
}
