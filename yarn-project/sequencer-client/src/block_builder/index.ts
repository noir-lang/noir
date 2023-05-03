import { L2Block } from '@aztec/types';
import { Proof } from '../prover/index.js';
import { ProcessedTx } from '../sequencer/processed_tx.js';
import { Fr } from '@aztec/foundation';

export interface BlockBuilder {
  buildL2Block(blockNumber: number, txs: ProcessedTx[], newL1ToL2Messages: Fr[]): Promise<[L2Block, Proof]>;
}
