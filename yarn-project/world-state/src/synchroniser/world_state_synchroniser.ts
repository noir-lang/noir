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
export interface WorldStateSynchroniser {
  /**
   * Starts the synchroniser.
   * @returns A promise that resolves once the initial sync is completed.
   */
  start(): void;

  /**
   * Returns the current status of the synchroniser.
   * @returns The current status of the synchroniser.
   */
  status(): Promise<WorldStateStatus>;

  /**
   * Stops the synchroniser.
   */
  stop(): Promise<void>;

  /**
   * Returns an instance of MerkleTreeOperations that will include uncommitted data.
   * @returns An instance of MerkleTreeOperations that will include uncommitted data.
   */
  getLatest(): MerkleTreeOperations;

  /**
   * Returns an instance of MerkleTreeOperations that will not include uncommitted data.
   * @returns An instance of MerkleTreeOperations that will not include uncommitted data.
   */
  getCommitted(): MerkleTreeOperations;
}
