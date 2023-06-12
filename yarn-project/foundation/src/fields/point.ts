import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { Fr } from './index.js';

/**
 * Represents a Point on an elliptic curve with x and y coordinates.
 * The Point class provides methods for creating instances from different input types,
 * converting instances to various output formats, and checking the equality of points.
 * It also contains constants for MODULUS, MAX_VALUE, SIZE_IN_BYTES, and ZERO point.
 * Each coordinate value should be within the range of 0 to MAX_VALUE inclusive.
 * Throws an error if the coordinate values are out of range.
 */
export class Point {
  static SIZE_IN_BYTES = 64;
  static ZERO = new Point(Buffer.alloc(Point.SIZE_IN_BYTES));
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = Point.MODULUS - 1n;

  /** Used to differentiate this class from AztecAddress */
  public readonly kind = 'point';

  constructor(
    /**
     * A buffer containing the x and y coordinates of the elliptic curve point.
     */
    /**
     * A buffer containing the x and y coordinates of the elliptic curve point.
     */
    public readonly buffer: Buffer,
  ) {
    const coordinateX = toBigIntBE(buffer.subarray(0, 32));
    const coordinateY = toBigIntBE(buffer.subarray(32, 64));
    if (coordinateX > Point.MAX_VALUE) {
      throw new Error(`Coordinate x out of range: ${coordinateX}.`);
    }
    if (coordinateY > Point.MAX_VALUE) {
      throw new Error(`Coordinate y out of range: ${coordinateY}.`);
    }
  }

  /**
   * Generate a random Point instance with coordinates within the valid range.
   * The coordinate values are generated as random Fr elements and converted to buffers
   * of size Point.SIZE_IN_BYTES before creating the Point instance.
   *
   * @returns A randomly generated Point instance.
   */
  static random() {
    // TODO is this a random point on the curve?
    return new Point(toBufferBE(Fr.random().value, Point.SIZE_IN_BYTES));
  }

  /**
   * Create a Point instance from a given buffer or BufferReader.
   * The input 'buffer' should have exactly 64 bytes representing the x and y coordinates.
   * Throws an error if the input length is invalid or coordinate values are out of range.
   *
   * @param buffer - The buffer or BufferReader containing the x and y coordinates of the point.
   * @returns A Point instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  /**
   * Create a Point instance from a hex-encoded string.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 128 hex characters representing the x and y coordinates.
   * Throws an error if the input length is invalid or coordinate values are out of range.
   *
   * @param address - The hex-encoded string representing the Point coordinates.
   * @returns A Point instance.
   */
  static fromString(address: string) {
    return new Point(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Convert the Point instance to a Buffer representation.
   * The output buffer's length will be equal to the `Point.SIZE_IN_BYTES` constant (64 bytes).
   * This method is useful for serialization and deserialization of the Point object.
   *
   * @returns A Buffer representation of the Point instance.
   */
  toBuffer() {
    return this.buffer;
  }

  /**
   * Convert the Point instance to a hexadecimal string representation.
   * The output string is prefixed with '0x' and consists of exactly 128 hex characters,
   * representing the concatenated x and y coordinates of the point.
   *
   * @returns A hex-encoded string representing the Point instance.
   */
  toString() {
    return '0x' + this.buffer.toString('hex');
  }

  /**
   * Generate a short string representation of the Point instance.
   * The returned string includes the first 10 and last 4 characters of the full string representation,
   * with '...' in between to indicate truncation. This is useful for displaying or logging purposes
   * when the full string representation may be too long.
   *
   * @returns A truncated string representation of the Point instance.
   */
  toShortString() {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }

  /**
   * Check if two Point instances are equal by comparing their buffer values.
   * Returns true if the buffer values are the same, and false otherwise.
   *
   * @param rhs - The Point instance to compare with the current instance.
   * @returns A boolean indicating whether the two Point instances are equal.
   */
  equals(rhs: Point) {
    return this.buffer.equals(rhs.buffer);
  }
}
