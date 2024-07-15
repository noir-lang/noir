import { poseidon2Hash, randomBoolean } from '../crypto/index.js';
import { BufferReader, FieldReader, serializeToBuffer } from '../serialize/index.js';
import { Fr } from './fields.js';

/**
 * Represents a Point on an elliptic curve with x and y coordinates.
 * The Point class provides methods for creating instances from different input types,
 * converting instances to various output formats, and checking the equality of points.
 */
export class Point {
  static ZERO = new Point(Fr.ZERO, Fr.ZERO, false);
  static SIZE_IN_BYTES = Fr.SIZE_IN_BYTES * 2;
  static COMPRESSED_SIZE_IN_BYTES = Fr.SIZE_IN_BYTES + 1;

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
    /**
     * Whether the point is at infinity
     */
    public readonly isInfinite: boolean,
  ) {
    // TODO(#7386): check if on curve
  }

  /**
   * Generate a random Point instance.
   *
   * @returns A randomly generated Point instance.
   */
  static random() {
    while (true) {
      try {
        return Point.fromXAndSign(Fr.random(), randomBoolean());
      } catch (e: any) {
        if (!(e instanceof NotOnCurveError)) {
          throw e;
        }
        // The random point is not on the curve - we try again
        continue;
      }
    }
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
    return new this(Fr.fromBuffer(reader), Fr.fromBuffer(reader), false);
  }

  /**
   * Create a Point instance from a compressed buffer.
   * The input 'buffer' should have exactly 33 bytes representing the x coordinate and the sign of the y coordinate.
   *
   * @param buffer - The buffer containing the x coordinate and the sign of the y coordinate.
   * @returns A Point instance.
   */
  static fromCompressedBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return this.fromXAndSign(Fr.fromBuffer(reader), reader.readBoolean());
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
    return [this.x, this.y, new Fr(this.isInfinite)];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new this(reader.readField(), reader.readField(), reader.readBoolean());
  }

  /**
   * Uses the x coordinate and isPositive flag (+/-) to reconstruct the point.
   * @dev The y coordinate can be derived from the x coordinate and the "sign" flag by solving the grumpkin curve
   * equation for y.
   * @param x - The x coordinate of the point
   * @param sign - The "sign" of the y coordinate - note that this is not a sign as is known in integer arithmetic.
   * Instead it is a boolean flag that determines whether the y coordinate is <= (Fr.MODULUS - 1) / 2
   * @returns The point as an array of 2 fields
   */
  static fromXAndSign(x: Fr, sign: boolean) {
    // Calculate y^2 = x^3 - 17
    const ySquared = x.square().mul(x).sub(new Fr(17));

    // Calculate the square root of ySquared
    const y = ySquared.sqrt();

    // If y is null, the x-coordinate is not on the curve
    if (y === null) {
      throw new NotOnCurveError();
    }

    const yPositiveBigInt = y.toBigInt() > (Fr.MODULUS - 1n) / 2n ? Fr.MODULUS - y.toBigInt() : y.toBigInt();
    const yNegativeBigInt = Fr.MODULUS - yPositiveBigInt;

    // Choose the positive or negative root based on isPositive
    const finalY = sign ? new Fr(yPositiveBigInt) : new Fr(yNegativeBigInt);

    // Create and return the new Point
    return new this(x, finalY, false);
  }

  /**
   * Returns the x coordinate and the sign of the y coordinate.
   * @dev The y sign can be determined by checking if the y coordinate is greater than half of the modulus.
   * @returns The x coordinate and the sign of the y coordinate.
   */
  toXAndSign(): [Fr, boolean] {
    return [this.x, this.y.toBigInt() <= (Fr.MODULUS - 1n) / 2n];
  }

  /**
   * Returns the contents of the point as BigInts.
   * @returns The point as BigInts
   */
  toBigInts() {
    return {
      x: this.x.toBigInt(),
      y: this.y.toBigInt(),
      isInfinite: this.isInfinite ? 1n : 0n,
    };
  }

  /**
   * Converts the Point instance to a Buffer representation of the coordinates.
   * @returns A Buffer representation of the Point instance.
   * @dev Note that toBuffer does not include the isInfinite flag and other serialization methods do (e.g. toFields).
   * This is because currently when we work with point as bytes we don't want to populate the extra bytes for
   * isInfinite flag because:
   * 1. Our Grumpkin BB API currently does not handle point at infinity,
   * 2. we use toBuffer when serializing notes and events and there we only work with public keys and point at infinity
   *   is not considered a valid public key and the extra byte would raise DA cost.
   */
  toBuffer() {
    if (this.isInfinite) {
      throw new Error('Cannot serialize infinite point without isInfinite flag');
    }
    const buf = serializeToBuffer([this.x, this.y]);
    if (buf.length !== Point.SIZE_IN_BYTES) {
      throw new Error(`Invalid buffer length for Point: ${buf.length}`);
    }
    return buf;
  }

  /**
   * Converts the Point instance to a compressed Buffer representation of the coordinates.
   * @returns A Buffer representation of the Point instance
   */
  toCompressedBuffer() {
    return serializeToBuffer(this.toXAndSign());
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

  toNoirStruct() {
    /* eslint-disable camelcase */
    return { x: this.x, y: this.y, is_infinite: this.isInfinite };
    /* eslint-enable camelcase */
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

  hash() {
    return poseidon2Hash(this.toFields());
  }

  /**
   * Check if this is point at infinity.
   * Check this is consistent with how bb is encoding the point at infinity
   */
  public get inf() {
    return this.x.isZero() && this.y.isZero() && this.isInfinite;
  }

  isOnGrumpkin() {
    // TODO: Check this against how bb handles curve check and infinity point check
    if (this.inf) {
      return true;
    }

    // p.y * p.y == p.x * p.x * p.x - 17
    const A = new Fr(17);
    const lhs = this.y.square();
    const rhs = this.x.square().mul(this.x).sub(A);
    return lhs.equals(rhs);
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

class NotOnCurveError extends Error {
  constructor() {
    super('The given x-coordinate is not on the Grumpkin curve');
    this.name = 'NotOnCurveError';
  }
}
