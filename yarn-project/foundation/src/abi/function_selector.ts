import { fromHex, toBigIntBE } from '../bigint-buffer/index.js';
import { keccak, randomBytes } from '../crypto/index.js';
import { type Fr } from '../fields/fields.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { FieldReader } from '../serialize/field_reader.js';
import { type ABIParameter } from './abi.js';
import { decodeFunctionSignature } from './decoder.js';
import { Selector } from './selector.js';

/* eslint-disable @typescript-eslint/no-unsafe-declaration-merging */

/** Function selector branding */
export interface FunctionSelector {
  /** Brand. */
  _branding: 'FunctionSelector';
}

/** A function selector is the first 4 bytes of the hash of a function signature. */
export class FunctionSelector extends Selector {
  /**
   * Checks if this function selector is equal to another.
   * @returns True if the function selectors are equal.
   */
  equals(fn: { name: string; parameters: ABIParameter[] }): boolean;
  equals(otherName: string, otherParams: ABIParameter[]): boolean;
  equals(other: FunctionSelector): boolean;
  equals(
    other: FunctionSelector | string | { name: string; parameters: ABIParameter[] },
    otherParams?: ABIParameter[],
  ): boolean {
    if (typeof other === 'string') {
      return this.equals(FunctionSelector.fromNameAndParameters(other, otherParams!));
    } else if (typeof other === 'object' && 'name' in other) {
      return this.equals(FunctionSelector.fromNameAndParameters(other.name, other.parameters));
    } else {
      return this.value === other.value;
    }
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer  or BufferReader to read from.
   * @returns The Selector.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const value = Number(toBigIntBE(reader.readBytes(Selector.SIZE)));
    return new FunctionSelector(value);
  }

  /**
   * Converts a field to selector.
   * @param fr - The field to convert.
   * @returns The selector.
   */
  static fromField(fr: Fr) {
    return new FunctionSelector(Number(fr.toBigInt()));
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return FunctionSelector.fromField(reader.readField());
  }

  /**
   * Creates a selector from a signature.
   * @param signature - Signature to generate the selector for (e.g. "transfer(field,field)").
   * @returns selector.
   */
  static fromSignature(signature: string) {
    // throw if signature contains whitespace
    if (/\s/.test(signature)) {
      throw new Error('Signature cannot contain whitespace');
    }
    return FunctionSelector.fromBuffer(keccak(Buffer.from(signature)).subarray(0, Selector.SIZE));
  }

  /**
   * Create a Selector instance from a hex-encoded string.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 64 hex characters.
   * Throws an error if the input length is invalid or address value is out of range.
   *
   * @param selector - The hex-encoded string representing the Selector.
   * @returns An Selector instance.
   */
  static fromString(selector: string) {
    const buf = fromHex(selector);
    if (buf.length !== Selector.SIZE) {
      throw new Error(`Invalid Selector length ${buf.length} (expected ${Selector.SIZE}).`);
    }
    return FunctionSelector.fromBuffer(buf);
  }

  /**
   * Creates an empty selector.
   * @returns An empty selector.
   */
  static empty() {
    return new FunctionSelector(0);
  }

  /**
   * Creates a function selector for a given function name and parameters.
   * @param name - The name of the function.
   * @param parameters - An array of ABIParameter objects, each containing the type information of a function parameter.
   * @returns A Buffer containing the 4-byte selector.
   */
  static fromNameAndParameters(args: { name: string; parameters: ABIParameter[] }): FunctionSelector;
  static fromNameAndParameters(name: string, parameters: ABIParameter[]): FunctionSelector;
  static fromNameAndParameters(
    nameOrArgs: string | { name: string; parameters: ABIParameter[] },
    maybeParameters?: ABIParameter[],
  ): FunctionSelector {
    const { name, parameters } =
      typeof nameOrArgs === 'string' ? { name: nameOrArgs, parameters: maybeParameters! } : nameOrArgs;
    const signature = decodeFunctionSignature(name, parameters);
    const selector = this.fromSignature(signature);
    // If using the debug logger here it kill the typing in the `server_world_state_synchronizer` and jest tests.
    // console.log(`selector for ${signature} is ${selector}`);
    return selector;
  }

  /**
   * Creates a random instance.
   */
  static random() {
    return FunctionSelector.fromBuffer(randomBytes(Selector.SIZE));
  }
}
