import { MerkleTreeOperations } from '../index.js';

/**
 * Defines the possible states of the world state synchroniser.
 */
export enum WorldStateRunningState {
  IDLE,
  SYNCHING,
  RUNNING,
  STOPPED,
}

/**
 * Defines the status of the world state synchroniser.
 */
export interface WorldStateStatus {
  /**
   * The current state of the world state synchroniser.
   */
  state: WorldStateRunningState;
  /**
   * The block number that the world state synchroniser is synced to.
   */
  syncedToL2Block: number;
}

/**
 * Defines the interface for a world state synchroniser.
 */
export interface WorldStateSynchroniser extends MerkleTreeOperations {
  start(): void;
  status(): Promise<WorldStateStatus>;
  stop(): Promise<void>;
}
