import { randomBytes } from '../crypto/random/index.js';

/**
 * The TxHash class represents a transaction hash in the form of a 32-byte buffer.
 * It provides methods to create a TxHash instance from different input formats such as Buffer, string, or random bytes,
 * as well as methods to compare, convert, and serialize the hash value.
 * This class ensures that the transaction hash is valid and properly formatted.
 */
export class TxHash {
  constructor(private buffer: Buffer) {
    if (buffer.length !== 32) {
      throw new Error('Invalid hash buffer.');
    }
  }

  /**
   * Create a TxHash instance from a given buffer.
   * The input 'buffer' should be a Buffer with exactly 32 bytes length.
   * Throws an error if the input buffer length is invalid.
   *
   * @param buffer - The Buffer representing the transaction hash.
   * @returns A TxHash instance.
   */
  static fromBuffer(buffer: Buffer) {
    return new TxHash(buffer);
  }

  /**
   * Deserialize a Buffer into a TxHash instance starting at the specified offset.
   * The function takes a buffer and an optional offset as input, slices the buffer
   * from the offset to offset + 32 bytes, and creates a new TxHash object using this slice.
   * Returns an object containing the created TxHash instance ('elem') and the number
   * of bytes advanced in the buffer ('adv'), which is always 32 for a valid deserialization.
   *
   * @param buffer - The input Buffer containing the serialized TxHash data.
   * @param offset - The optional starting position within the buffer to begin deserialization. Defaults to 0.
   * @returns An object with properties 'elem' (TxHash instance) and 'adv' (number of bytes advanced).
   */
  static deserialize(buffer: Buffer, offset: number) {
    return { elem: new TxHash(buffer.slice(offset, offset + 32)), adv: 32 };
  }

  /**
   * Create a TxHash instance from a hex-encoded string.
   * The input 'hash' should be prefixed with '0x' or not, and have exactly 64 hex characters.
   * Throws an error if the input length is invalid or the hash value is out of range.
   *
   * @param hash - The hex-encoded string representing the transaction hash.
   * @returns A TxHash instance.
   */
  public static fromString(hash: string) {
    return new TxHash(Buffer.from(hash.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Generate a random TxHash instance with a buffer of 32 random bytes.
   * This function utilizes the 'randomBytes' function from the crypto library to generate
   * a buffer filled with cryptographically secure random bytes, which is then used to create
   * the new TxHash instance.
   *
   * @returns A random TxHash instance.
   */
  public static random() {
    return new TxHash(randomBytes(32));
  }

  /**
   * Compares the current TxHash instance with the provided TxHash instance.
   * Returns true if their buffer contents are equal, otherwise returns false.
   *
   * @param rhs - The TxHash instance to compare with the current instance.
   * @returns A boolean indicating whether the two TxHash instances have identical buffer contents.
   */
  equals(rhs: TxHash) {
    return this.toBuffer().equals(rhs.toBuffer());
  }

  /**
   * Converts the current TxHash instance to a Buffer.
   * The resulting buffer will have a length of 32 bytes.
   *
   * @returns A Buffer representation of the transaction hash.
   */
  toBuffer() {
    return this.buffer;
  }

  /**
   * Converts the TxHash instance to a hex-encoded string representation.
   * The resulting string is prefixed with '0x' and contains exactly 64 hex characters.
   *
   * @returns A string representing the TxHash in hex format.
   */
  toString() {
    return `0x${this.toBuffer().toString('hex')}`;
  }
}
