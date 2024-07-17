import { Fr, Point } from '@aztec/circuits.js';
import { randomBytes, sha256Trunc } from '@aztec/foundation/crypto';

/**
 * Represents an individual encrypted log entry.
 */
export class EncryptedL2NoteLog {
  constructor(
    /** The encrypted data contents of the log. */
    public readonly data: Buffer,
  ) {}

  get length(): number {
    return this.data.length;
  }

  /**
   * Serializes log to a buffer.
   * @returns A buffer containing the serialized log.
   */
  public toBuffer(): Buffer {
    return this.data;
  }

  /** Returns a JSON-friendly representation of the log. */
  public toJSON(): object {
    return {
      data: this.data.toString('hex'),
    };
  }

  /** Converts a plain JSON object into an instance. */
  public static fromJSON(obj: any) {
    return new EncryptedL2NoteLog(Buffer.from(obj.data, 'hex'));
  }

  /**
   * Deserializes log from a buffer.
   * @param buffer - The buffer containing the log.
   * @returns Deserialized instance of `Log`.
   */
  public static fromBuffer(data: Buffer): EncryptedL2NoteLog {
    return new EncryptedL2NoteLog(data);
  }

  /**
   * Calculates hash of serialized logs.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public hash(): Buffer {
    const preimage = this.toBuffer();
    return sha256Trunc(preimage);
  }

  public getSiloedHash(): Buffer {
    return this.hash();
  }

  /**
   * Crates a random log.
   * @returns A random log.
   */
  public static random(): EncryptedL2NoteLog {
    const randomEphPubKey = Point.random();
    const randomLogContent = randomBytes(144 - Point.COMPRESSED_SIZE_IN_BYTES);
    const data = Buffer.concat([Fr.random().toBuffer(), randomLogContent, randomEphPubKey.toCompressedBuffer()]);
    return new EncryptedL2NoteLog(data);
  }

  public static empty() {
    return new EncryptedL2NoteLog(Buffer.alloc(0));
  }
}
