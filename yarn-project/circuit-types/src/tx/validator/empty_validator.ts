import { type AnyTx, type TxValidator } from './tx_validator.js';

export class EmptyTxValidator<T extends AnyTx = AnyTx> implements TxValidator<T> {
  public validateTxs(txs: T[]): Promise<[validTxs: T[], invalidTxs: T[]]> {
    return Promise.resolve([txs, []]);
  }
}
