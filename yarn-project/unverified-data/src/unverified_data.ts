import { serializeBufferToVector, deserializeBufferFromVector } from '@aztec/foundation/src/serialize/free_funcs.ts';
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
