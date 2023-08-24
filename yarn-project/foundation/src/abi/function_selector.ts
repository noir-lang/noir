import { ABIParameter } from '@aztec/foundation/abi';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { keccak } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * A function selector is the first 4 bytes of the hash of a function signature.
 */
export class FunctionSelector {
  /**
   * The size of the function selector in bytes.
   */
  public static SIZE = 4;

  constructor(/** number representing the function selector */ public value: number) {
    if (value > 2 ** (FunctionSelector.SIZE * 8) - 1) {
      throw new Error(`Function selector must fit in ${FunctionSelector.SIZE} bytes.`);
    }
  }

  /**
   * Checks if the function selector is empty (all bytes are 0).
   * @returns True if the function selector is empty (all bytes are 0).
   */
  public isEmpty(): boolean {
    return this.value === 0;
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return toBufferBE(BigInt(this.value), FunctionSelector.SIZE);
  }

  /**
   * Serialize as a hex string.
   * @returns The string.
   */
  toString(): string {
    return this.toBuffer().toString('hex');
  }

  /**
   * Checks if this function selector is equal to another.
   * @param other - The other function selector.
   * @returns True if the function selectors are equal.
   */
  equals(other: FunctionSelector): boolean {
    return this.value === other.value;
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer  or BufferReader to read from.
   * @returns The FunctionSelector.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionSelector {
    const reader = BufferReader.asReader(buffer);
    const value = Number(toBigIntBE(reader.readBytes(FunctionSelector.SIZE)));
    return new FunctionSelector(value);
  }

  /**
   * Returns a new field with the same contents as this EthAddress.
   *
   * @returns An Fr instance.
   */
  public toField() {
    return new Fr(this.value);
  }

  /**
   * Converts a field to function selector.
   * @param fr - The field to convert.
   * @returns The function selector.
   */
  static fromField(fr: Fr): FunctionSelector {
    return new FunctionSelector(Number(fr.value));
  }

  /**
   * Creates a function selector from a signature.
   * @param signature - Signature of the function to generate the selector for (e.g. "transfer(field,field)").
   * @returns Function selector.
   */
  static fromSignature(signature: string): FunctionSelector {
    return FunctionSelector.fromBuffer(keccak(Buffer.from(signature)).subarray(0, FunctionSelector.SIZE));
  }

  /**
   * Creates a function selector for a given function name and parameters.
   * @param name - The name of the function.
   * @param parameters - An array of ABIParameter objects, each containing the type information of a function parameter.
   * @returns A Buffer containing the 4-byte function selector.
   */
  static fromNameAndParameters(name: string, parameters: ABIParameter[]) {
    const signature = name === 'constructor' ? name : `${name}(${parameters.map(p => p.type.kind).join(',')})`;
    return FunctionSelector.fromSignature(signature);
  }

  /**
   * Creates an empty function selector.
   * @returns An empty function selector.
   */
  static empty(): FunctionSelector {
    return new FunctionSelector(0);
  }
}
