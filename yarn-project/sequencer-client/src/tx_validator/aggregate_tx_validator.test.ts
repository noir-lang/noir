import { type AnyTx, Tx, type TxHash, type TxValidator, mockTx } from '@aztec/circuit-types';

import { AggregateTxValidator } from './aggregate_tx_validator.js';

describe('AggregateTxValidator', () => {
  it('allows txs that pass all validation', async () => {
    const txs = [mockTx(0), mockTx(1), mockTx(2), mockTx(3), mockTx(4)];
    const agg = new AggregateTxValidator(
      new TxDenyList(txs[0].getTxHash(), txs[1].getTxHash()),
      new TxDenyList(txs[2].getTxHash(), txs[3].getTxHash()),
    );

    await expect(agg.validateTxs(txs)).resolves.toEqual([[txs[4]], [txs[0], txs[1], txs[2], txs[3]]]);
  });

  class TxDenyList implements TxValidator<AnyTx> {
    denyList: Set<string>;
    constructor(...txHashes: TxHash[]) {
      this.denyList = new Set(txHashes.map(hash => hash.toString()));
    }

    validateTxs(txs: AnyTx[]): Promise<[AnyTx[], AnyTx[]]> {
      return Promise.resolve([
        txs.filter(tx => !this.denyList.has(Tx.getHash(tx).toString())),
        txs.filter(tx => this.denyList.has(Tx.getHash(tx).toString())),
      ]);
    }
  }
});
