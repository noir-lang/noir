import { UnverifiedData } from './unverified_data.js';

/**
 * Interface of classes allowing for the retrieval of unverified data.
 */
export interface UnverifiedDataSource {
  /**
   * Gets the L2 block number associated with the latest unverified data.
   * @returns The L2 block number associated with the latest unverified data.
   */
  getLatestUnverifiedDataBlockNum(): Promise<number>;

  /**
   * Gets the `take` amount of unverified data starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first `unverifiedData` to be returned.
   * @param take - The number of `unverifiedData` to return.
   * @returns The requested `unverifiedData`.
   */
  getUnverifiedData(from: number, take: number): Promise<UnverifiedData[]>;

  /**
   * Starts the unverified data source.
   * @param blockUntilSynced - If true, blocks until the data source has fully synced.
   * @returns A promise signalling completion of the start process.
   */
  start(blockUntilSynced: boolean): Promise<void>;

  /**
   * Stops the unverified data source.
   * @returns A promise signalling completion of the stop process.
   */
  stop(): Promise<void>;
}
