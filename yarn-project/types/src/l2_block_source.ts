import { EthAddress } from '@aztec/circuits.js';

import { L2Block } from './l2_block.js';

/**
 * Interface of classes allowing for the retrieval of L2 blocks.
 */
export interface L2BlockSource {
  /**
   * Method to fetch the rollup contract address at the base-layer.
   * @returns The rollup address.
   */
  getRollupAddress(): Promise<EthAddress>;

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Gets an l2 block. If a negative number is passed, the block returned is the most recent.
   * @param number - The block number to return (inclusive).
   * @returns The requested L2 block.
   */
  getL2Block(number: number): Promise<L2Block | undefined>;

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The maximum number of blocks to return.
   * @returns The requested L2 blocks.
   */
  getL2Blocks(from: number, limit: number): Promise<L2Block[]>;

  /**
   * Starts the L2 block source.
   * @param blockUntilSynced - If true, blocks until the data source has fully synced.
   * @returns A promise signalling completion of the start process.
   */
  start(blockUntilSynced: boolean): Promise<void>;

  /**
   * Stops the L2 block source.
   * @returns A promise signalling completion of the stop process.
   */
  stop(): Promise<void>;
}
