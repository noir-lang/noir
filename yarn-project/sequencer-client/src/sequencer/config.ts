/**
 * The sequencer configuration.
 */
export interface SequencerConfig {
  /**
   * The number of ms to wait between polling for pending txs.
   */
  transactionPollingInterval: number;
  /**
   * Up to how many txs to include a block.
   */
  maxTxsPerBlock: number;
}
