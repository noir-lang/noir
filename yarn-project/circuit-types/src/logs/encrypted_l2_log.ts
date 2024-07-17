import { Fr, Point } from '@aztec/circuits.js';
import { randomBytes, sha256Trunc } from '@aztec/foundation/crypto';

/**
 * Represents an individual encrypted event log entry.
 */
export class EncryptedL2Log {
  constructor(public readonly data: Buffer, public readonly maskedContractAddress: Fr) {}

  // We do not 'count' the maskedContractAddress in .length, as this method is called to calculate ciphertext length
  get length(): number {
    return this.data.length;
  }

  /**
   * Serializes log to a buffer.
   * @returns A buffer containing the serialized log.
   */
  public toBuffer(): Buffer {
    return Buffer.concat([this.maskedContractAddress.toBuffer(), this.data]);
  }

  /** Returns a JSON-friendly representation of the log. */
  public toJSON(): object {
    return {
      data: this.data.toString('hex'),
      maskedContractAddress: this.maskedContractAddress.toString(),
    };
  }

  /** Converts a plain JSON object into an instance. */
  public static fromJSON(obj: any) {
    return new EncryptedL2Log(Buffer.from(obj.data, 'hex'), Fr.fromString(obj.maskedContractAddress));
  }

  /**
   * Deserializes log from a buffer.
   * @param buffer - The buffer containing the log.
   * @returns Deserialized instance of `Log`.
   */
  public static fromBuffer(data: Buffer): EncryptedL2Log {
    return new EncryptedL2Log(data.subarray(32), new Fr(data.subarray(0, 32)));
  }

  /**
   * Calculates hash of serialized logs.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public hash(): Buffer {
    return sha256Trunc(this.data);
  }

  /**
   * Calculates siloed hash of serialized encryptedlogs.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public getSiloedHash(): Buffer {
    const hash = this.hash();
    return sha256Trunc(Buffer.concat([this.maskedContractAddress.toBuffer(), hash]));
  }

  /**
   * Crates a random log.
   * @returns A random log.
   */
  public static random(): EncryptedL2Log {
    const randomEphPubKey = Point.random();
    const randomLogContent = randomBytes(144 - Point.COMPRESSED_SIZE_IN_BYTES);
    const data = Buffer.concat([Fr.random().toBuffer(), randomLogContent, randomEphPubKey.toCompressedBuffer()]);
    return new EncryptedL2Log(data, Fr.random());
  }
}
