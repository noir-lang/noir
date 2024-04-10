import { type ProcessedTx, type Tx } from '@aztec/circuit-types';

export type AnyTx = Tx | ProcessedTx;

export interface TxValidator<T extends AnyTx = AnyTx> {
  validateTxs(txs: T[]): Promise<[validTxs: T[], invalidTxs: T[]]>;
}
