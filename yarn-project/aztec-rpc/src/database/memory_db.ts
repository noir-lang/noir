import { TxHash } from '@aztec/tx';

import { MemoryContractDataSource } from '../contract_data_source/index.js';
import { Database } from './database.js';
import { TxDao } from './tx_dao.js';

export class MemoryDB extends MemoryContractDataSource implements Database {
  private txs: TxDao[] = [];

  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txs.find(tx => tx.txHash.equals(txHash)));
  }
}
