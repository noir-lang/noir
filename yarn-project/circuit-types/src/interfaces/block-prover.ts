import { type Fr, type GlobalVariables, type Proof } from '@aztec/circuits.js';

import { type L2Block } from '../l2_block.js';
import { type ProcessedTx } from '../tx/processed_tx.js';

export enum PROVING_STATUS {
  SUCCESS,
  FAILURE,
}

export type ProvingSuccess = {
  status: PROVING_STATUS.SUCCESS;
};

export type ProvingFailure = {
  status: PROVING_STATUS.FAILURE;
  reason: string;
};

export type ProvingResult = ProvingSuccess | ProvingFailure;

export type ProvingTicket = {
  provingPromise: Promise<ProvingResult>;
};

export type BlockResult = {
  block: L2Block;
  proof: Proof;
};

/**
 * The interface to the block prover.
 * Provides the ability to generate proofs and build rollups.
 */
export interface BlockProver {
  /**
   * Cancels any block that is currently being built and prepares for a new one to be built
   * @param numTxs - The complete size of the block, must be a power of 2
   * @param globalVariables - The global variables for this block
   * @param l1ToL2Messages - The set of L1 to L2 messages to be included in this block
   * @param emptyTx - An instance of an empty transaction to be used in this block
   */
  startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    l1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket>;

  /**
   * Add a processed transaction to the current block
   * @param tx - The transaction to be added
   */
  addNewTx(tx: ProcessedTx): Promise<void>;

  /**
   * Cancels the block currently being proven. Proofs already bring built may continue but further proofs should not be started.
   */
  cancelBlock(): void;

  /**
   * Performs the final archive tree insertion for this block and returns the L2Block and Proof instances
   */
  finaliseBlock(): Promise<BlockResult>;

  /**
   * Mark the block as having all the transactions it is going to contain.
   * Will pad the block to it's complete size with empty transactions and prove all the way to the root rollup.
   */
  setBlockCompleted(): Promise<void>;
}
