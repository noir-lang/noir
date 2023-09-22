import { MerkleTreeOperations } from '../index.js';

/**
 * Defines the possible states of the world state synchronizer.
 */
export enum WorldStateRunningState {
  IDLE,
  SYNCHING,
  RUNNING,
  STOPPED,
}

/**
 * Defines the status of the world state synchronizer.
 */
export interface WorldStateStatus {
  /**
   * The current state of the world state synchronizer.
   */
  state: WorldStateRunningState;
  /**
   * The block number that the world state synchronizer is synced to.
   */
  syncedToL2Block: number;
}

/**
 * Defines the interface for a world state synchronizer.
 */
export interface WorldStateSynchronizer {
  /**
   * Starts the synchronizer.
   * @returns A promise that resolves once the initial sync is completed.
   */
  start(): void;

  /**
   * Returns the current status of the synchronizer.
   * @returns The current status of the synchronizer.
   */
  status(): Promise<WorldStateStatus>;

  /**
   * Stops the synchronizer.
   */
  stop(): Promise<void>;

  /**
   * Forces an immediate sync to an optionally provided minimum block number
   * @param minBlockNumber - The minimum block number that we must sync to
   * @returns A promise that resolves once the sync has completed.
   */
  syncImmediate(minBlockNumber?: number): Promise<void>;

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
