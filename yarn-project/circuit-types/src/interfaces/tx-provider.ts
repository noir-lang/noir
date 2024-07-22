import { type Tx } from '../tx/tx.js';
import { type TxHash } from '../tx/tx_hash.js';

/** Provider for transaction objects given their hash. */
export interface TxProvider {
  /**
   * Returns a transaction given its hash if available.
   * @param txHash - The hash of the transaction, used as an ID.
   * @returns The transaction, if found, 'undefined' otherwise.
   */
  getTxByHash(txHash: TxHash): Promise<Tx | undefined>;
}
