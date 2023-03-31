import { AztecAddress, Fr } from '@aztec/foundation';
import { TxHash } from '@aztec/tx';

import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { NoteDao } from './note_dao.js';
import { TxAuxDataDao } from './tx_aux_data_dao.js';
import { TxDao } from './tx_dao.js';

export class MemoryDB extends MemoryContractDatabase implements Database {
  private txs: TxDao[] = [];
  private txAccountData: TxAuxDataDao[] = [];
  private notes: NoteDao[] = [];

  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txs.find(tx => tx.txHash.equals(txHash)));
  }

  public addNote(note: NoteDao) {
    this.notes.push(note);
    return Promise.resolve();
  }

  public getNotes(contractAddress: AztecAddress, storageSlot: Fr): Promise<NoteDao[]> {
    return Promise.resolve(
      this.notes.filter(
        note =>
          note.contractAddress.equals(contractAddress) && note.contractSlot.toBuffer().equals(storageSlot.toBuffer()),
      ),
    );
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

  public addTxAuxDataDao(txAuxDataDao: TxAuxDataDao): Promise<void> {
    this.txAccountData.push(txAuxDataDao);
    return Promise.resolve();
  }

  public getStorageAt(contract: AztecAddress, storageSlot: Fr): TxAuxDataDao | undefined {
    return this.txAccountData.find(
      txAccountData =>
        txAccountData.contractAddress.equals(contract) &&
        txAccountData.storageSlot.toBuffer().equals(storageSlot.toBuffer()),
    );
  }
}
