import { type Fr, type GlobalVariables, type Proof } from '@aztec/circuits.js';

import { type L2Block } from '../l2_block.js';
import { type ProcessedTx } from '../tx/processed_tx.js';

export enum PROVING_STATUS {
  SUCCESS,
  FAILURE,
}

export type ProvingSuccess = {
  status: PROVING_STATUS.SUCCESS;
  block: L2Block;
  proof: Proof;
};

export type ProvingFailure = {
  status: PROVING_STATUS.FAILURE;
  reason: string;
};

export type ProvingResult = ProvingSuccess | ProvingFailure;

export type ProvingTicket = {
  provingPromise: Promise<ProvingResult>;
};

/**
 * The interface to the block prover.
 * Provides the ability to generate proofs and build rollups.
 */
export interface BlockProver {
  startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    l1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket>;

  addNewTx(tx: ProcessedTx): Promise<void>;
}
