/**
 * Data retrieved from logs
 */
export type DataRetrieval<T> = {
  /**
   * Blocknumber of the last L1 block from which we obtained data.
   */
  lastProcessedL1BlockNumber: bigint;
  /**
   * The data returned.
   */
  retrievedData: T[];
};

/**
 * Data retrieved from logs
 */
export type SingletonDataRetrieval<T> = {
  /**
   * Blocknumber of the last L1 block from which we obtained data.
   */
  lastProcessedL1BlockNumber: bigint;
  /**
   * The data returned.
   */
  retrievedData: T;
};
