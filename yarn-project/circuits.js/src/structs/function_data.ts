import { BufferReader } from '@aztec/foundation/serialize';
import { serializeToBuffer } from '../utils/serialize.js';

const FUNCTION_SELECTOR_LENGTH = 4;

/**
 * Function description for circuit.
 * @see abis/function_data.hpp
 */
export class FunctionData {
  constructor(public functionSelector: Buffer, public isPrivate = true, public isConstructor = false) {
    if (functionSelector.byteLength !== FUNCTION_SELECTOR_LENGTH) {
      throw new Error(
        `Function selector must be ${FUNCTION_SELECTOR_LENGTH} bytes long, got ${functionSelector.byteLength} bytes.`,
      );
    }
  }
  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.functionSelector, this.isPrivate, this.isConstructor);
  }

  /**
   * Returns whether this instance is empty.
   * @returns True if the function selector is zero.
   */
  isEmpty() {
    return this.functionSelector.equals(Buffer.alloc(FUNCTION_SELECTOR_LENGTH, 0));
  }

  public static empty(args?: { isPrivate?: boolean; isConstructor?: boolean }) {
    return new FunctionData(Buffer.alloc(FUNCTION_SELECTOR_LENGTH, 0), args?.isPrivate, args?.isConstructor);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionData {
    const reader = BufferReader.asReader(buffer);
    return new FunctionData(reader.readBytes(FUNCTION_SELECTOR_LENGTH), reader.readBoolean(), reader.readBoolean());
  }
}
