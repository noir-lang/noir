import { L2Block } from './index.js';

/**
 * Interface of classes allowing for the retrieval of L2 blocks.
 */
export interface L2BlockSource {
  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  getLatestBlockNum(): Promise<number>;

  /**
   * Gets the `take` amount of L2 blocks starting from `from`.
   * @param from - If of the first rollup to return (inclusive).
   * @param take - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  getL2Blocks(from: number, take: number): Promise<L2Block[]>;

  /**
   * Starts the L2 block source.
   * @returns A promise signalling completion of the start process.
   */
  start(): Promise<void>;

  /**
   * Stops the L2 block source.
   * @returns A promise signalling completion of the stop process.
   */
  stop(): Promise<void>;
}
