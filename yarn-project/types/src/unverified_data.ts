import { serializeBufferToVector, deserializeBufferFromVector } from '@aztec/foundation';
import { randomBytes } from 'crypto';
/**
 * Data container of unverified data corresponding to one L2 block.
 */
export class UnverifiedData {
  /**
   * Constructs an object containing unverified data.
   * @param dataChunks - Chunks of unverified data corresponding to individual pieces of information (e.g. encrypted preimages).
   */
  constructor(public readonly dataChunks: Buffer[]) {}

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

  public static random(numChunks: number): UnverifiedData {
    const chunks: Buffer[] = [];
    for (let i = 0; i < numChunks; i++) {
      chunks.push(randomBytes(144));
    }
    return new UnverifiedData(chunks);
  }
}
