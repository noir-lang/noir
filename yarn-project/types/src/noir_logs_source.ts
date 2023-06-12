import { NoirLogs } from './noir_logs.js';

/**
 * Interface of classes allowing for the retrieval of encrypted logs.
 */
export interface NoirLogsSource {
  /**
   * Gets the `take` amount of encrypted logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first `encryptedLogs` to be returned.
   * @param take - The number of `encryptedLogs` to return.
   * @returns The requested `encryptedLogs`.
   */
  getEncryptedLogs(from: number, take: number): Promise<NoirLogs[]>;

  /**
   * Starts the encrypted logs source.
   * @param blockUntilSynced - If true, blocks until the data source has fully synced.
   * @returns A promise signalling completion of the start process.
   */
  start(blockUntilSynced: boolean): Promise<void>;

  /**
   * Stops the encrypted logs source.
   * @returns A promise signalling completion of the stop process.
   */
  stop(): Promise<void>;
}
