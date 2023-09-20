import { AztecAddress } from '../aztec-address/index.js';
import { toBigIntBE, toBufferBE, toHex } from '../bigint-buffer/index.js';
import { randomBytes } from '../crypto/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

/**
 * Fr represents a field of integers modulo the prime number MODULUS.
 * It provides utility functions to work with elements in this field, such as conversions between different representations and checks for equality and zero values. The elements can be serialized to and deserialized from byte buffers or strings.
 * Some use cases include working with cryptographic operations and finite fields.
 */
export class Fr {
  static ZERO = new Fr(0n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = Fr.MODULUS - 1n;
  static SIZE_IN_BYTES = 32;

  /**
   * The numeric value of the field element as a bigint.
   */
  public readonly value: bigint;

  constructor(value: boolean | bigint | number | Fr | AztecAddress) {
    const isFr = (value: boolean | bigint | number | Fr | AztecAddress): value is Fr | AztecAddress =>
      !!(value as Fr).toBigInt;
    this.value = isFr(value) ? value.toBigInt() : BigInt(value);
    if (this.value > Fr.MAX_VALUE) {
      throw new Error(`Fr out of range ${value}.`);
    }
  }

  /**
   * Generates a random Fr or Fq instance with a value modulo the respective class' MODULUS.
   * This method uses randomBytes to generate a random 32-byte buffer, converts it to a bigint
   * and takes the modulus of the result with the class' MODULUS constant.
   *
   * @returns A new Fr or Fq instance with a random value.
   */
  static random() {
    const r = toBigIntBE(randomBytes(Fr.SIZE_IN_BYTES)) % Fr.MODULUS;
    return new Fr(r);
  }

  /**
   * Returns a new zero-value field.
   * @returns A new zero-value field.
   */
  static zero() {
    return new Fr(0n);
  }

  /**
   * Create an instance of the corresponding class (Fr or Fq) from a Buffer or a BufferReader.
   * Reads 'SIZE_IN_BYTES' bytes from the given Buffer or BufferReader and constructs an instance with the decoded value.
   * If the input is a Buffer, it is internally converted to a BufferReader before reading.
   * Throws an error if the input length is invalid or the decoded value is out of range.
   *
   * @param buffer - The Buffer or BufferReader containing the bytes representing the value.
   * @returns An instance of the corresponding class (Fr or Fq) with the decoded value.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new Fr(toBigIntBE(reader.readBytes(Fr.SIZE_IN_BYTES)));
  }

  /**
   * Create a Fr instance from a hex-encoded string.
   * The input 'address' can be either prefixed with '0x' or not, and should have exactly 64 hex characters.
   * Throws an error if the input length is invalid or the address value is out of range.
   *
   * @param address - The hex-encoded string representing the field element.
   * @returns A Fr instance.
   */
  static fromString(address: string) {
    return Fr.fromBuffer(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Converts the value of the instance to a buffer with a specified length.
   * The method uses the provided value and size in bytes to create a buffer representation
   * of the numeric value. This can be useful for serialization and communication purposes.
   *
   * @returns A buffer representing the instance's value.
   */
  toBuffer() {
    return toBufferBE(this.value, Fr.SIZE_IN_BYTES);
  }

  /**
   * Converts the value of the Fr or Fq class instance to a hexadecimal string.
   * The resulting string is prefixed with '0x' and represents the bigint value
   * in base 16.
   *
   * @param padTo32 - Whether to pad the string to 32 bytes.
   * @returns A hex-encoded string representing the value of the class instance.
   */
  toString(padTo32 = false): `0x${string}` {
    return toHex(this.value, padTo32);
  }

  /**
   * Retrieves the underlying bigint.
   * This method mostly exists to match user expectations, as value is already public.
   * @returns The underlying bigint.
   */
  public toBigInt(): bigint {
    return this.value;
  }

  /**
   * Returns a shortened string representation of the Fr value, formatted with '0x' prefix and ellipsis in the middle.
   * The resulting string has first 10 characters (including '0x') and last 4 characters of the full hexadecimal value.
   *
   * @returns A shorter, human-readable string representation of the Fr value.
   */
  toShortString() {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }

  /**
   * Checks if the provided Fr instance is equal to the current instance.
   * Two instances are considered equal if their 'value' properties are the same.
   *
   * @param rhs - The Fr instance to compare with the current instance.
   * @returns A boolean indicating whether the two instances are equal.
   */
  equals(rhs: Fr) {
    return this.value === rhs.value;
  }

  /**
   * Check if the instance value is zero.
   * The method returns true if the value of the instance is 0n (zero in BigInt representation),
   * otherwise, it returns false.
   *
   * @returns A boolean indicating whether the instance value is zero or not.
   */
  isZero() {
    return this.value === 0n;
  }

  /**
   * Converts the current value of the Fq or Fr instance to a friendly JSON representation.
   * The output will be a hexadecimal string prefixed with '0x'.
   *
   * @returns A '0x' prefixed hexadecimal string representing the current value.
   */
  toFriendlyJSON() {
    return this.toString();
  }

  /** Returns self. */
  toField() {
    return this;
  }
}

/**
 * Fq represents a field element in a prime finite field with modulus defined by the constant MODULUS.
 * It provides methods for creating, manipulating, and comparing field elements, as well as converting
 * them to/from different data types like Buffers and hex strings. Field elements are used in various
 * cryptographic protocols and operations, such as elliptic curve cryptography.
 *
 * @example
 * const fqElem = new Fq(BigInt("123456789"));
 * const randomFqElem = Fq.random();
 * const fromBufferFqElem = Fq.fromBuffer(buffer);
 */
export class Fq {
  static MODULUS = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47n;
  static MAX_VALUE = Fr.MODULUS - 1n;
  static SIZE_IN_BYTES = 32;

  constructor(
    /**
     * The element's value as a bigint in the finite field.
     */
    public readonly value: bigint,
  ) {
    if (value > Fq.MAX_VALUE) {
      throw new Error(`Fq out of range ${value}.`);
    }
  }

  /**
   * Generates a random Fr or Fq instance with a value within the range of their respective modulus.
   * The random value is generated from a byte array of length equal to SIZE_IN_BYTES, then truncated
   * to the appropriate modulus before creating the new Fr or Fq instance.
   *
   * @returns A new Fr or Fq instance with a randomly generated value.
   */
  static random() {
    const r = toBigIntBE(randomBytes(64)) % Fq.MODULUS;
    return new this(r);
  }

  /**
   * Create an instance of the calling class (Fr or Fq) from a given buffer or BufferReader.
   * Reads SIZE_IN_BYTES from the provided buffer and converts it to a bigint, then creates a new instance
   * with that value. Throws an error if the value is out of range for the calling class.
   *
   * @param buffer - The input buffer or BufferReader containing the bytes representing the value.
   * @returns An instance of the calling class (Fr or Fq) initialized with the bigint value.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(toBigIntBE(reader.readBytes(this.SIZE_IN_BYTES)));
  }

  /**
   * Converts the bigint value of the instance to a Buffer representation.
   * The output buffer has a fixed size, determined by the 'SIZE_IN_BYTES' constant.
   *
   * @returns A Buffer containing the byte representation of the instance's value.
   */
  toBuffer() {
    return toBufferBE(this.value, Fq.SIZE_IN_BYTES);
  }

  /**
   * Converts the Fq value to a hexadecimal string representation.
   * The resulting string is prefixed with '0x' and contains the exact number of hex characters required
   * to represent the numeric value of this instance.
   *
   * @returns A hexadecimal string representing the Fq value.
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
   * Converts the value of the Fr or Fq instance to a friendly JSON format.
   * The output is a hexadecimal string representation of the value with '0x' prefix.
   *
   * @returns A string representing the value in the JSON format.
   */
  toFriendlyJSON() {
    return this.toString();
  }

  /** Returns self. */
  toField() {
    return this;
  }
}
