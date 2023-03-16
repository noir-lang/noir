import { L2Block } from './l2_block/l2_block.js';

/**
 * Describes sync status of the archiver.
 */
export interface SyncStatus {
  /**
   * The height of the L2 block that the archiver is synced to.
   */
  syncedToBlock: number;
  /**
   * Maximum height of a L2 block processed by the rollup contract.
   */
  latestBlock: number;
}

/**
 * Interface of classes allowing for the retrieval of L2 blocks.
 */
export interface L2BlockSource {
  /**
   * Gets the sync status of the L2 block source.
   * @returns The sync status of the L2 block source.
   */
  getSyncStatus(): Promise<SyncStatus>;

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  getLatestBlockNum(): number;

  /**
   * Gets the `take` amount of L2 blocks starting from `from`.
   * @param from - If of the first rollup to return (inclusive).
   * @param take - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  getL2Blocks(from: number, take: number): L2Block[];
}
