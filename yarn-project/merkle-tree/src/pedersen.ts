import {
  pedersenCompress,
  pedersenCompressInputs,
  pedersenGetHash,
  pedersenGetHashTree,
} from '@aztec/barretenberg.js/crypto';

import { WasmWrapper } from '@aztec/foundation/wasm';
import { Hasher } from './hasher.js';

/**
 * A helper class encapsulating Pedersen hash functionality.
 */
export class Pedersen implements Hasher {
  constructor(private wasm: WasmWrapper) {}

  /**
   * Compresses two 32-byte hashes.
   * @param lhs - The first hash.
   * @param rhs - The second hash.
   * @returns The new 32-byte hash.
   */
  public compress(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return pedersenCompress(this.wasm, lhs, rhs);
  }

  /**
   * Compresses an array of buffers.
   * @param inputs - The array of buffers to compress.
   * @returns The resulting 32-byte hash.
   */
  public compressInputs(inputs: Buffer[]): Buffer {
    return pedersenCompressInputs(this.wasm, inputs);
  }

  /**
   * Get a 32-byte pedersen hash from a buffer.
   * @param data - The data buffer.
   * @returns The resulting hash buffer.
   */
  public hashToField(data: Uint8Array): Buffer {
    return pedersenGetHash(this.wasm, Buffer.from(data));
  }

  /**
   * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of
   * nodes that define a merkle tree.
   *
   * E.g.
   * Input:  [1][2][3][4]
   * Output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)].
   *
   * @param leaves - The 32 byte pedersen leaves.
   * @returns A tree represented by an array.
   */
  public hashToTree(leaves: Buffer[]): Promise<Buffer[]> {
    return Promise.resolve(pedersenGetHashTree(this.wasm, leaves));
  }
}
