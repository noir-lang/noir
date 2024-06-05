/** Provides up to which block has been synced by different components. */
export type SyncStatus = {
  /** Up to which block has been synched for blocks and txs. */
  blocks: number;
  /** Up to which block has been synched for notes, indexed by each account address being monitored. */
  notes: Record<string, number>;
};
