import type { BlockAttestation, BlockProposal, Gossipable } from '@aztec/circuit-types';

import type { ENR } from '@chainsafe/enr';
import type { PeerId } from '@libp2p/interface';
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
   * @param message - The message to be propagated.
   */
  propagate<T extends Gossipable>(message: T): void;

  // Leaky abstraction: fix https://github.com/AztecProtocol/aztec-packages/issues/7963
  registerBlockReceivedCallback(callback: (block: BlockProposal) => Promise<BlockAttestation>): void;
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
   * Runs findRandomNode query.
   */
  runRandomNodesQuery(): Promise<void>;

  /**
   * Checks if the given peer is a bootstrap peer.
   * @param peerId - The peer ID to check.
   * @returns True if the peer is a bootstrap peer.
   */
  isBootstrapPeer(peerId: PeerId): boolean;

  /**
   * Event emitted when a new peer is discovered.
   */
  on(event: 'peer:discovered', listener: (enr: ENR) => void): this;
  emit(event: 'peer:discovered', enr: ENR): boolean;

  getStatus(): PeerDiscoveryState;
}
