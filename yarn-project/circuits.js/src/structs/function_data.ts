import { FunctionAbi, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { BufferReader } from '@aztec/foundation/serialize';

import { ContractFunctionDao } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';

/**
 * Function description for circuit.
 * @see abis/function_data.hpp
 */
export class FunctionData {
  constructor(
    /**
     * Function selector of the function being called.
     */
    public selector: FunctionSelector,
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
  ) {}

  static fromAbi(abi: FunctionAbi | ContractFunctionDao): FunctionData {
    return new FunctionData(
      FunctionSelector.fromNameAndParameters(abi.name, abi.parameters),
      abi.isInternal,
      abi.functionType === FunctionType.SECRET,
      abi.name === 'constructor',
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.selector, this.isInternal, this.isPrivate, this.isConstructor);
  }

  /**
   * Returns whether this instance is empty.
   * @returns True if the function selector is zero.
   */
  isEmpty() {
    return this.selector.isEmpty();
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
      FunctionSelector.empty(),
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
      reader.readObject(FunctionSelector),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
    );
  }
}
