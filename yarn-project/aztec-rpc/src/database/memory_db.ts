import { AztecAddress, Fr } from '@aztec/foundation';
import { TxHash } from '@aztec/types';
import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { TxAuxDataDao } from './tx_aux_data_dao.js';
import { TxDao } from './tx_dao.js';

export class MemoryDB extends MemoryContractDatabase implements Database {
  private txTable: TxDao[] = [];
  private txAuxDataTable: TxAuxDataDao[] = [];

  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txTable.find(tx => tx.txHash.equals(txHash)));
  }

  public getTxsByAddress(from: AztecAddress) {
    return Promise.resolve(this.txTable.filter(tx => tx.from.equals(from)));
  }

  public addTx(tx: TxDao) {
    const index = this.txTable.findIndex(t => t.txHash.equals(tx.txHash));
    if (index === -1) {
      this.txTable.push(tx);
    } else {
      this.txTable[index] = tx;
    }
    return Promise.resolve();
  }

  public async addTxs(txs: TxDao[]) {
    await Promise.all(txs.map(tx => this.addTx(tx)));
  }

  public addTxAuxData(txAuxDataDao: TxAuxDataDao) {
    this.txAuxDataTable.push(txAuxDataDao);
    return Promise.resolve();
  }

  public addTxAuxDataBatch(txAuxDataDao: TxAuxDataDao[]) {
    this.txAuxDataTable.push(...txAuxDataDao);
    return Promise.resolve();
  }

  public getTxAuxData(contract: AztecAddress, storageSlot: Fr) {
    const res = this.txAuxDataTable.filter(
      txAuxData =>
        txAuxData.contractAddress.equals(contract) && txAuxData.storageSlot.toBuffer().equals(storageSlot.toBuffer()),
    );
    return Promise.resolve(res);
  }
}
