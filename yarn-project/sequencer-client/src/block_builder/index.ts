import { L2Block, Tx } from '@aztec/types';
import { Proof } from '../prover/index.js';

export interface BlockBuilder {
  buildL2Block(blockNumber: number, txs: Tx[]): Promise<[L2Block, Proof]>;
}
