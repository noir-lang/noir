import { type FunctionAbi, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { FUNCTION_DATA_LENGTH, GeneratorIndex } from '../constants.gen.js';
import { type ContractFunctionDao } from '../types/contract_function_dao.js';

/** Function description for circuit. */
export class FunctionData {
  constructor(
    /** Function selector of the function being called. */
    public selector: FunctionSelector,
    /** Indicates whether the function is private or public. */
    public isPrivate: boolean,
  ) {}

  static fromAbi(abi: FunctionAbi | ContractFunctionDao): FunctionData {
    return new FunctionData(
      FunctionSelector.fromNameAndParameters(abi.name, abi.parameters),
      abi.functionType === FunctionType.SECRET,
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.selector, this.isPrivate);
  }

  toFields(): Fr[] {
    const fields = [this.selector.toField(), new Fr(this.isPrivate)];
    if (fields.length !== FUNCTION_DATA_LENGTH) {
      throw new Error(
        `Invalid number of fields for FunctionData. Expected ${FUNCTION_DATA_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
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
    /** Indicates whether the function is private or public. */
    isPrivate?: boolean;
  }): FunctionData {
    return new FunctionData(FunctionSelector.empty(), args?.isPrivate ?? false);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of FunctionData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionData {
    const reader = BufferReader.asReader(buffer);
    return new FunctionData(reader.readObject(FunctionSelector), reader.readBoolean());
  }

  static fromFields(fields: Fr[] | FieldReader): FunctionData {
    const reader = FieldReader.asReader(fields);

    const selector = FunctionSelector.fromFields(reader);
    const isPrivate = reader.readBoolean();

    return new FunctionData(selector, isPrivate);
  }

  hash(): Fr {
    return pedersenHash(this.toFields(), GeneratorIndex.FUNCTION_DATA);
  }
}
