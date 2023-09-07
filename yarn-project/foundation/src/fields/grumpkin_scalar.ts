import { toBigIntBE, toBufferBE, toHex } from '../bigint-buffer/index.js';
import { randomBytes } from '../crypto/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { Fr } from './fields.js';

/**
 * Represents a field element in a prime finite field with modulus defined by the constant MODULUS.
 * @remarks Called GrumpkinScalar because it is used to represent elements in Grumpkin's scalar field as defined in
 *          the Aztec Yellow Paper.
 */
export class GrumpkinScalar {
  static MODULUS = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47n;
  static MAX_VALUE = GrumpkinScalar.MODULUS - 1n;
  static SIZE_IN_BYTES = 32;

  // The following constants are used to split a GrumpkinScalar into two Fr elements.
  private static HIGH_SHIFT = BigInt((GrumpkinScalar.SIZE_IN_BYTES / 2) * 8);
  private static LOW_MASK = (1n << GrumpkinScalar.HIGH_SHIFT) - 1n;

  constructor(
    /**
     * The element's value as a bigint in the finite field.
     */
    public readonly value: bigint,
  ) {
    if (value > GrumpkinScalar.MAX_VALUE) {
      throw new Error(`GrumpkinScalar out of range ${value}.`);
    }
  }

  get low(): Fr {
    return new Fr(this.value & GrumpkinScalar.LOW_MASK);
  }

  get high(): Fr {
    return new Fr(this.value >> GrumpkinScalar.HIGH_SHIFT);
  }

  /**
   * Deserialize a grumpkin scalar serialized in 2 Fr.
   * @param high - The high Fr element.
   * @param low - The low Fr element.
   * @returns A GrumpkinScalar instance with the value of the two Fr elements.
   */
  static fromHighLow(high: Fr, low: Fr): GrumpkinScalar {
    return new GrumpkinScalar((high.value << GrumpkinScalar.HIGH_SHIFT) + low.value);
  }

  /**
   * Generates a random GrumpkinScalar.
   *
   * @returns A new GrumpkinScalar instance with a randomly generated value.
   */
  static random() {
    const r = toBigIntBE(randomBytes(64)) % GrumpkinScalar.MODULUS;
    return new this(r);
  }

  /**
   * Create an instance of GrumpkinScalar from a given buffer or BufferReader.
   * @remarks Reads SIZE_IN_BYTES from the provided buffer and converts it to a bigint, then creates a new instance
   * with that value. Throws an error if the value is out of range for the calling class.
   *
   * @param buffer - The input buffer or BufferReader containing the bytes representing the value.
   * @returns A GrumpkinScalar instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(toBigIntBE(reader.readBytes(this.SIZE_IN_BYTES)));
  }

  /**
   * Like fromBuffer, but reduces the value modulo MODULUS.
   *
   * @param buffer - The Buffer or BufferReader containing the bytes representing the value.
   * @returns GrumpkinScalar with the decoded value.
   */
  static fromBufferWithReduction(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const value = toBigIntBE(reader.readBytes(GrumpkinScalar.SIZE_IN_BYTES)) % GrumpkinScalar.MODULUS;
    return new GrumpkinScalar(value);
  }

  /**
   * Create a GrumpkinScalar instance from a hex-encoded string.
   * The input 'address' can be either prefixed with '0x' or not, and should have exactly 64 hex characters.
   * Throws an error if the input length is invalid or the address value is out of range.
   *
   * @param address - The hex-encoded string representing the field element.
   * @returns A GrumpkinScalar instance.
   */
  static fromString(address: string) {
    return GrumpkinScalar.fromBuffer(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Converts the bigint value of the instance to a Buffer representation.
   * The output buffer has a fixed size, determined by the 'SIZE_IN_BYTES' constant.
   *
   * @returns A Buffer containing the byte representation of the instance's value.
   */
  toBuffer() {
    return toBufferBE(this.value, GrumpkinScalar.SIZE_IN_BYTES);
  }

  /**
   * Converts the GrumpkinScalar value to a hexadecimal string representation.
   * The resulting string is prefixed with '0x' and contains the exact number of hex characters required
   * to represent the numeric value of this instance.
   *
   * @returns A hexadecimal string representing the GrumpkinScalar value.
   */
  toString() {
    return toHex(this.value);
  }

  /**
   * Check if the value of the current instance is zero.
   * This function compares the internal 'value' property with 0n (BigInt representation of zero).
   * Returns true if the value is zero, otherwise returns false.
   *
   * @returns A boolean indicating whether the value is zero or not.
   */
  isZero() {
    return this.value === 0n;
  }

  /**
   * Converts the value of the GrumpkinScalar instance to a friendly JSON format.
   * The output is a hexadecimal string representation of the value with '0x' prefix.
   *
   * @returns A string representing the value in the JSON format.
   */
  toFriendlyJSON() {
    return this.toString();
  }
}
