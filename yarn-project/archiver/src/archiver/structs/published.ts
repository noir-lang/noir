/** Extends a type with L1 published info (block number, hash, and timestamp) */
export type L1Published<T> = {
  data: T;
  l1: L1PublishedData;
};

export type L1PublishedData = {
  blockNumber: bigint;
  timestamp: bigint;
  blockHash: string;
};
