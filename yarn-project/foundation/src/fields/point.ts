import { BufferReader, FieldReader } from '../serialize/index.js';
import { Fr } from './fields.js';

/**
 * Represents a Point on an elliptic curve with x and y coordinates.
 * The Point class provides methods for creating instances from different input types,
 * converting instances to various output formats, and checking the equality of points.
 */
export class Point {
  static ZERO = new Point(Fr.ZERO, Fr.ZERO);
  static SIZE_IN_BYTES = Fr.SIZE_IN_BYTES * 2;

  /** Used to differentiate this class from AztecAddress */
  public readonly kind = 'point';

  constructor(
    /**
     * The point's x coordinate
     */
    public readonly x: Fr,
    /**
     * The point's y coordinate
     */
    public readonly y: Fr,
  ) {}

  /**
   * Generate a random Point instance.
   *
   * @returns A randomly generated Point instance.
   */
  static random() {
    // TODO is this a random point on the curve?
    return new Point(Fr.random(), Fr.random());
  }

  /**
   * Create a Point instance from a given buffer or BufferReader.
   * The input 'buffer' should have exactly 64 bytes representing the x and y coordinates.
   *
   * @param buffer - The buffer or BufferReader containing the x and y coordinates of the point.
   * @returns A Point instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(Fr.fromBuffer(reader.readBytes(32)), Fr.fromBuffer(reader.readBytes(32)));
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
    return this.fromBuffer(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Returns the contents of the point as an array of 2 fields.
   * @returns The point as an array of 2 fields
   */
  toFields() {
    return [this.x, this.y];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new this(reader.readField(), reader.readField());
  }

  /**
   * Returns the contents of the point as BigInts.
   * @returns The point as BigInts
   */
  toBigInts() {
    return {
      x: this.x.toBigInt(),
      y: this.y.toBigInt(),
    };
  }

  /**
   * Converts the Point instance to a Buffer representation of the coordinates.
   * The outputs buffer length will be 64, the length of both coordinates not represented as fields.
   * @returns A Buffer representation of the Point instance.
   */
  toBuffer() {
    return Buffer.concat([this.x.toBuffer(), this.y.toBuffer()]);
  }

  /**
   * Convert the Point instance to a hexadecimal string representation.
   * The output string is prefixed with '0x' and consists of exactly 128 hex characters,
   * representing the concatenated x and y coordinates of the point.
   *
   * @returns A hex-encoded string representing the Point instance.
   */
  toString() {
    return '0x' + this.toBuffer().toString('hex');
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
    return this.x.equals(rhs.x) && this.y.equals(rhs.y);
  }

  isZero() {
    return this.x.isZero() && this.y.isZero();
  }
}

/**
 * Does this object look like a point?
 * @param obj - Object to test if it is a point.
 * @returns Whether it looks like a point.
 */
export function isPoint(obj: object): obj is Point {
  if (!obj) {
    return false;
  }
  const point = obj as Point;
  return point.kind === 'point' && point.x !== undefined && point.y !== undefined;
}
