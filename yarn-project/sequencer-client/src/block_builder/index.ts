import { L2Block } from '@aztec/types';
import { Proof } from '../prover/index.js';
import { ProcessedTx } from '../sequencer/processed_tx.js';
import { Fr } from '@aztec/foundation';

/**
 * Assembles an L2Block from a set of processed transactions.
 */
export interface BlockBuilder {
  /**
   * Creates a new L2Block with the given number, containing the set of processed txs.
   * Note that the number of txs need to be a power of two.
   * @param blockNumber - Number of the block to assemble.
   * @param txs - Processed txs to include.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @returns The new L2 block along with its proof from the root circuit.
   */
  buildL2Block(blockNumber: number, txs: ProcessedTx[], newL1ToL2Messages: Fr[]): Promise<[L2Block, Proof]>;
}
