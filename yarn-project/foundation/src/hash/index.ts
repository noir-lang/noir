import { randomBytes } from '@aztec/foundation/crypto';
import { BufferReader, deserializeBigInt, serializeBigInt } from '@aztec/foundation/serialize';

import { type Fr } from '../fields/fields.js';

/**
 * A class representing a hash.
 */
export class BaseHashType {
  /**
   * The size of the hash in bytes.
   */
  public static SIZE = 32;

  /**
   * HashType with value zero.
   */
  public static ZERO = new BaseHashType(Buffer.alloc(BaseHashType.SIZE));

  constructor(
    /**
     * The buffer containing the hash.
     */
    public buffer: Buffer,
  ) {
    if (buffer.length !== BaseHashType.SIZE) {
      throw new Error(`Expected buffer to have length ${BaseHashType.SIZE} but was ${buffer.length}`);
    }
  }

  /**
   * Returns the raw buffer of the hash.
   * @returns The buffer containing the hash.
   */
  public toBuffer() {
    return this.buffer;
  }

  /**
   * Creates a HashType from a buffer.
   * @param buffer - The buffer to create from.
   * @returns A new HashType object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new BaseHashType(reader.readBytes(BaseHashType.SIZE));
  }

  /**
   * Checks if this hash and another hash are equal.
   * @param hash - A hash to compare with.
   * @returns True if the hashes are equal, false otherwise.
   */
  public equals(hash: BaseHashType): boolean {
    return this.buffer.equals(hash.buffer);
  }

  /**
   * Returns true if this hash is zero.
   * @returns True if this hash is zero.
   */
  public isZero(): boolean {
    return this.buffer.equals(Buffer.alloc(32, 0));
  }

  /**
   * Convert this hash to a hex string.
   * @returns The hex string.
   */
  public toString() {
    return this.buffer.toString('hex');
  }

  /**
   * Convert this hash to a big int.
   * @returns The big int.
   */
  public toBigInt() {
    return deserializeBigInt(this.buffer, 0, BaseHashType.SIZE).elem;
  }
  /**
   * Creates a tx hash from a bigint.
   * @param hash - The tx hash as a big int.
   * @returns The HashType.
   */
  public static fromBigInt(hash: bigint) {
    return new BaseHashType(serializeBigInt(hash, BaseHashType.SIZE));
  }
  public static fromField(hash: Fr) {
    return new BaseHashType(serializeBigInt(hash.toBigInt()));
  }

  /**
   * Converts this hash from a buffer of 28 bytes.
   * Verifies the input is 28 bytes.
   * @param buffer - The 28 byte buffer to construct from.
   * @returns A HashType created from the input buffer with 4 bytes 0 padding at the front.
   */
  public static fromBuffer28(buffer: Buffer) {
    if (buffer.length != 28) {
      throw new Error(`Expected HashType input buffer to be 28 bytes`);
    }
    const padded = Buffer.concat([Buffer.alloc(this.SIZE - 28), buffer]);
    return new BaseHashType(padded);
  }

  /**
   * Converts a string into a HashType object.
   * @param str - The TX hash in string format.
   * @returns A new HashType object.
   */
  public static fromString(str: string): BaseHashType {
    return new BaseHashType(Buffer.from(str, 'hex'));
  }

  /**
   * Generates a random HashType.
   * @returns A new HashType object.
   */
  public static random(): BaseHashType {
    return new BaseHashType(Buffer.from(randomBytes(BaseHashType.SIZE)));
  }
}
