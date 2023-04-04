import { toBigIntBE } from '@aztec/foundation';
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
    // Determine total length of all chunks
    let totalLength = 0;
    for (const buffer of this.dataChunks) {
      totalLength += buffer.length;
    }

    // Create a new buffer with enough space to hold all serialized chunks
    const serializedBuffer = Buffer.allocUnsafe(totalLength + 4 * this.dataChunks.length);

    // Serialize each buffer into the new buffer with prefix
    let offset = 0;
    for (const buffer of this.dataChunks) {
      // Write the length of the buffer as a 4-byte prefix
      serializedBuffer.writeInt32BE(buffer.length, offset);
      offset += 4;

      // Write the contents of the buffer
      buffer.copy(serializedBuffer, offset);
      offset += buffer.length;
    }

    return serializedBuffer;
  }

  public static fromBuffer(buf: Buffer): UnverifiedData {
    let currIndex = 0;
    const chunks: Buffer[] = [];
    while (currIndex < buf.length) {
      const nextChunkLength = Number(toBigIntBE(buf.slice(currIndex, currIndex + 4)));
      currIndex += 4;
      const nextChunk = buf.slice(currIndex, currIndex + nextChunkLength);
      currIndex += nextChunkLength;
      chunks.push(nextChunk);
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
