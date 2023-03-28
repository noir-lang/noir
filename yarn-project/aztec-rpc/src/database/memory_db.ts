import { TxHash } from '@aztec/tx';

import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { TxDao } from './tx_dao.js';

export class MemoryDB extends MemoryContractDatabase implements Database {
  private txs: TxDao[] = [];

  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txs.find(tx => tx.txHash.equals(txHash)));
  }

  public addOrUpdateTx(tx: TxDao): Promise<void> {
    const index = this.txs.findIndex(t => t.txHash.equals(tx.txHash));
    if (index === -1) {
      this.txs.push(tx);
    } else {
      this.txs[index] = tx;
    }
    return Promise.resolve();
  }
}
