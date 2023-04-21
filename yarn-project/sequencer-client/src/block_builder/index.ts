import { L2Block } from '@aztec/types';
import { Proof } from '../prover/index.js';
import { ProcessedTx } from '../sequencer/processed_tx.js';

export interface BlockBuilder {
  buildL2Block(blockNumber: number, txs: ProcessedTx[]): Promise<[L2Block, Proof]>;
}
