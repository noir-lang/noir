import { type Tx, type TxHash } from '@aztec/circuit-types';

/**
 * The interface for a P2P service implementation.
 */
export interface P2PService {
  /**
   * Starts the service.
   * @returns An empty promise.
   */
  start(): Promise<void>;

  /**
   * Stops the service.
   * @returns An empty promise.
   */
  stop(): Promise<void>;

  /**
   * Called to have the given transaction propagated through the P2P network.
   * @param tx - The transaction to be propagated.
   */
  propagateTx(tx: Tx): void;

  /**
   * Called upon receipt of settled transactions.
   * @param txHashes - The hashes of the settled transactions.
   */
  settledTxs(txHashes: TxHash[]): void;
}
