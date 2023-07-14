import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

/**
 * AztecAddress represents a 32-byte address in the Aztec Protocol. It provides methods to create, manipulate, and
 * compare addresses. The maximum value of an address is determined by the field modulus and all instances of AztecAddress
 * should have a value less than or equal to this max value. This class also provides helper functions to convert
 * addresses from strings, buffers, and other formats.
 */
export class AztecAddress {
  static SIZE_IN_BYTES = 32;
  static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE_IN_BYTES));
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = AztecAddress.MODULUS - 1n;

  constructor(
    /**
     * The buffer field.
     */
    public readonly buffer: Buffer,
  ) {
    const value = toBigIntBE(buffer);
    if (value > AztecAddress.MAX_VALUE) {
      throw new Error(`AztecAddress out of range ${value}.`);
    }
  }

  /**
   * Generates a random AztecAddress instance, using the Fr field (a finite field) to create a random value
   * within a valid range and then converting it into a Buffer of a fixed size in bytes.
   *
   * @returns A new AztecAddress instance with a random value.
   */
  static random() {
    return new AztecAddress(toBufferBE(Fr.random().value, AztecAddress.SIZE_IN_BYTES));
  }

  /**
   * Creates an AztecAddress instance from a given buffer or BufferReader.
   * If the input is a Buffer, it wraps it in a BufferReader before processing.
   * Throws an error if the input length is not equal to the expected size.
   *
   * @param buffer - The input buffer or BufferReader containing the address data.
   * @returns - A new AztecAddress instance with the extracted address data.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  /**
   * Create an AztecAddress instance from a hex-encoded string.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 64 hex characters.
   * Throws an error if the input length is invalid or address value is out of range.
   *
   * @param address - The hex-encoded string representing the Aztec address.
   * @returns An AztecAddress instance.
   */
  static fromString(address: string) {
    const buf = Buffer.from(address.replace(/^0x/i, ''), 'hex');
    if (buf.length !== AztecAddress.SIZE_IN_BYTES) {
      throw new Error(`Invalid length ${buf.length}.`);
    }
    return new AztecAddress(buf);
  }

  /**
   * Creates an AztecAddress from a bigint.
   * The provided value must be within the range of a field.
   * @param address - The bigint representation of the address.
   * @returns An AztecAddress instance.
   */
  static fromBigInt(address: bigint) {
    return new AztecAddress(toBufferBE(address, AztecAddress.SIZE_IN_BYTES));
  }

  /**
   * Converts the AztecAddress instance into a Buffer.
   * This method should be used when encoding the address for storage, transmission or serialization purposes.
   *
   * @returns A Buffer representation of the AztecAddress instance.
   */
  toBuffer() {
    return this.buffer;
  }

  /**
   * Convert the AztecAddress to a hexadecimal string representation, with a "0x" prefix.
   * The resulting string will have a length of 66 characters (including the prefix).
   *
   * @returns A hexadecimal string representation of the AztecAddress.
   */
  toString(): `0x${string}` {
    return `0x${this.buffer.toString('hex')}`;
  }

  /**
   * Returns a shortened string representation of the AztecAddress, displaying only the first 10 characters and last 4 characters.
   * This is useful for human-readable displays where the full address is not necessary.
   *
   * @returns A shortened string representation of the address.
   */
  toShortString() {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }

  /**
   * Returns this address as a field element.
   * @returns A field element with the same value as the address.
   */
  toField() {
    return Fr.fromBuffer(this.toBuffer());
  }

  /**
   * Returns this address as a bigint. Useful for creating maps indexed by addresses.
   * @returns A bigint with the same value as the address.
   */
  toBigInt() {
    return toBigIntBE(this.buffer);
  }

  /**
   * Determines if this AztecAddress instance is equal to the given AztecAddress instance.
   * Equality is based on the content of their respective buffers.
   *
   * @param rhs - The AztecAddress instance to compare against.
   * @returns True if the buffers of both instances are equal, false otherwise.
   */
  equals(rhs: AztecAddress) {
    return this.buffer.equals(rhs.buffer);
  }

  /**
   * Checks if the AztecAddress is zero.
   *
   * @returns Returns true if the AztecAddress is equal to the zero address, otherwise returns false.
   */
  isZero() {
    return this.equals(AztecAddress.ZERO);
  }

  /**
   * Friendly representation for debugging purposes.
   *
   * @returns A hex string representing the address.
   */
  toFriendlyJSON() {
    return this.toString();
  }
}
