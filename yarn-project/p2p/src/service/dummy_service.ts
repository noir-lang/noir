import { Tx, TxHash } from '@aztec/types';

import { P2PService } from './service.js';

/**
 * A dummy implementation of the P2P Service.
 */
export class DummyP2PService implements P2PService {
  /**
   * Starts the dummy implementation.
   * @returns A resolved promise.
   */
  public start() {
    return Promise.resolve();
  }

  /**
   * Stops the dummy imaplementation.
   * @returns A resolved promise.
   */
  public stop() {
    return Promise.resolve();
  }

  /**
   * Called to have the given transaction propagated through the P2P network.
   * @param _ - The transaction to be propagated.
   */
  public propagateTx(_: Tx) {}

  /**
   * Called upon receipt of settled transactions.
   * @param _ - The hashes of the settled transactions.
   */
  public settledTxs(_: TxHash[]) {}
}
