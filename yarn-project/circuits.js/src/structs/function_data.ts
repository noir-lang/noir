import { FunctionAbi, FunctionType, generateFunctionSelector } from '@aztec/foundation/abi';
import { BufferReader, deserializeUInt32, numToUInt32BE } from '@aztec/foundation/serialize';

import { ContractFunctionDao } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';

const FUNCTION_SELECTOR_LENGTH = 4;

/**
 * Function description for circuit.
 * @see abis/function_data.hpp
 */
export class FunctionData {
  /**
   * Function selector of the function being called.
   */
  public functionSelectorBuffer: Buffer;
  constructor(
    functionSelector: Buffer | number,
    /**
     * Indicates whether the function is only callable by self or not.
     */
    public isInternal: boolean,
    /**
     * Indicates whether the function is private or public.
     */
    public isPrivate: boolean,
    /**
     * Indicates whether the function is a constructor.
     */
    public isConstructor: boolean,
  ) {
    if (functionSelector instanceof Buffer) {
      if (functionSelector.byteLength !== FUNCTION_SELECTOR_LENGTH) {
        throw new Error(
          `Function selector must be ${FUNCTION_SELECTOR_LENGTH} bytes long, got ${functionSelector.byteLength} bytes.`,
        );
      }
      this.functionSelectorBuffer = functionSelector;
    } else {
      // create a new numeric buffer with 4 bytes
      this.functionSelectorBuffer = numToUInt32BE(functionSelector);
    }
  }

  static fromAbi(abi: FunctionAbi | ContractFunctionDao): FunctionData {
    return new FunctionData(
      generateFunctionSelector(abi.name, abi.parameters),
      abi.isInternal,
      abi.functionType === FunctionType.SECRET,
      abi.name === 'constructor',
    );
  }

  // For serialization, must match function_selector name in C++ and return as number
  // TODO(AD) somehow remove this cruft, probably by using a buffer selector in C++
  get functionSelector(): number {
    return deserializeUInt32(this.functionSelectorBuffer).elem;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.functionSelectorBuffer, this.isInternal, this.isPrivate, this.isConstructor);
  }

  /**
   * Returns whether this instance is empty.
   * @returns True if the function selector is zero.
   */
  isEmpty() {
    return this.functionSelectorBuffer.equals(Buffer.alloc(FUNCTION_SELECTOR_LENGTH, 0));
  }

  /**
   * Returns a new instance of FunctionData with zero function selector.
   * @param args - Arguments to pass to the constructor.
   * @returns A new instance of FunctionData with zero function selector.
   */
  public static empty(args?: {
    /**
     * Indicates whether the function is only callable by self or not.
     */
    isInternal?: boolean;
    /**
     * Indicates whether the function is private or public.
     */
    isPrivate?: boolean;
    /**
     * Indicates whether the function is a constructor.
     */
    isConstructor?: boolean;
  }): FunctionData {
    return new FunctionData(
      Buffer.alloc(FUNCTION_SELECTOR_LENGTH, 0),
      args?.isInternal ?? false,
      args?.isPrivate ?? false,
      args?.isConstructor ?? false,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of FunctionData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionData {
    const reader = BufferReader.asReader(buffer);
    return new FunctionData(
      reader.readBytes(FUNCTION_SELECTOR_LENGTH),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
    );
  }
}
