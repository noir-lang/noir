/**
 * Data container of unverified data corresponding to one L2 block.
 */
export class UnverifiedData {
  /**
   * Constructs an object containing unverified data.
   * @param dataChunks - Chunks of unverified data corresponding to individual pieces of information (e.g. encrypted preimages).
   */
  constructor(public readonly dataChunks: Buffer[]) {}
}
