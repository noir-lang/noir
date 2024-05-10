import type { Tx, TxHash } from '@aztec/circuit-types';

import type { ENR } from '@chainsafe/enr';
import type EventEmitter from 'events';

export enum PeerDiscoveryState {
  RUNNING = 'running',
  STOPPED = 'stopped',
}

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

/**
 * The interface for a peer discovery service implementation.
 */
export interface PeerDiscoveryService extends EventEmitter {
  /**
   * Starts the service.
   * */
  start(): Promise<void>;

  /**
   * Stops the service.
   * */
  stop(): Promise<void>;

  /**
   * Gets all peers.
   * @returns An array of peer ENRs.
   */
  getAllPeers(): ENR[];

  /**
   * Event emitted when a new peer is discovered.
   */
  on(event: 'peer:discovered', listener: (enr: ENR) => void): this;
  emit(event: 'peer:discovered', enr: ENR): boolean;

  getStatus(): PeerDiscoveryState;
}
