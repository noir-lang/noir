import { GlobalVariables, Proof } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { L2Block } from '@aztec/types';

import { ProcessedTx } from '../sequencer/processed_tx.js';

/**
 * Assembles an L2Block from a set of processed transactions.
 */
export interface BlockBuilder {
  /**
   * Creates a new L2Block with the given number, containing the set of processed txs.
   * Note that the number of txs need to be a power of two.
   * @param globalVariables - Global variables to include in the block.
   * @param txs - Processed txs to include.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @returns The new L2 block along with its proof from the root circuit.
   */
  buildL2Block(
    globalVariables: GlobalVariables,
    txs: ProcessedTx[],
    newL1ToL2Messages: Fr[],
  ): Promise<[L2Block, Proof]>;
}
