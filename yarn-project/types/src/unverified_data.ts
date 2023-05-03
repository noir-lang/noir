import { serializeBufferToVector, deserializeBufferFromVector } from '@aztec/foundation';
import { randomBytes } from 'crypto';

/**
 * Data container of unverified data corresponding to one L2 block.
 */
export class UnverifiedData {
  constructor(
    /**
     * Chunks of unverified data corresponding to individual pieces of information (e.g. Encrypted preimages).
     */
    public readonly dataChunks: Buffer[],
  ) {}

  /**
   * Serializes unverified data into a buffer.
   * @returns A buffer containing the serialized unverified data.
   */
  public toBuffer(): Buffer {
    // Serialize each buffer into the new buffer with prefix
    const serializedChunks = this.dataChunks.map(buffer => serializeBufferToVector(buffer));
    // Concatenate all serialized chunks into a single buffer
    const serializedBuffer = Buffer.concat(serializedChunks);

    return serializedBuffer;
  }

  /**
   * Creates a new UnverifiedData object by concatenating multiple ones.
   * @param datas - The individual data objects to concatenate.
   * @returns A new UnverifiedData object whose chunks are the concatenation of the chunks.
   */
  public static join(datas: UnverifiedData[]): UnverifiedData {
    return new UnverifiedData(datas.flatMap(chunk => chunk.dataChunks));
  }

  /**
   * Deserializes unverified data from a buffer.
   * @param buf - The buffer containing the serialized unverified data.
   * @returns A new UnverifiedData object.
   */
  public static fromBuffer(buf: Buffer): UnverifiedData {
    let currIndex = 0;
    const chunks: Buffer[] = [];
    while (currIndex < buf.length) {
      const { elem, adv } = deserializeBufferFromVector(buf, currIndex);
      chunks.push(elem);
      currIndex += adv;
    }
    if (currIndex !== buf.length) {
      throw new Error(
        `Unverified data buffer was not fully consumed. Consumed ${currIndex + 1} bytes. Total length: ${
          buf.length
        } bytes.`,
      );
    }
    return new UnverifiedData(chunks);
  }

  /**
   * Creates a new UnverifiedData object with `numChunks` random data.
   * @param numChunks - The number of chunks to create.
   * @returns A new UnverifiedData object.
   */
  public static random(numChunks: number): UnverifiedData {
    const chunks: Buffer[] = [];
    for (let i = 0; i < numChunks; i++) {
      chunks.push(randomBytes(144));
    }
    return new UnverifiedData(chunks);
  }
}
