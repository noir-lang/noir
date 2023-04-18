import { AztecAddress, Fr, Point } from '@aztec/foundation';
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

  public addTxAuxDataBatch(txAuxDataDaos: TxAuxDataDao[]) {
    this.txAuxDataTable.push(...txAuxDataDaos);
    return Promise.resolve();
  }

  public getTxAuxData(contract: AztecAddress, storageSlot: Fr) {
    const res = this.txAuxDataTable.filter(
      txAuxData =>
        txAuxData.contractAddress.equals(contract) && txAuxData.storageSlot.toBuffer().equals(storageSlot.toBuffer()),
    );
    return Promise.resolve(res);
  }

  public removeNullifiedTxAuxData(nullifiers: Fr[], account: Point) {
    const nullifierSet = new Set(nullifiers.map(nullifier => nullifier.toString()));
    const [remaining, removed] = this.txAuxDataTable.reduce(
      (acc: [TxAuxDataDao[], TxAuxDataDao[]], txAuxData) => {
        const nullifier = txAuxData.nullifier.toString();
        if (txAuxData.account.equals(account) && nullifierSet.has(nullifier)) {
          acc[1].push(txAuxData);
        } else {
          acc[0].push(txAuxData);
        }
        return acc;
      },
      [[], []],
    );

    this.txAuxDataTable = remaining;

    return Promise.resolve(removed);
  }
}
