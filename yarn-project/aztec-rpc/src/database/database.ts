import { AztecAddress, Fr } from '@aztec/foundation';
import { TxHash } from '@aztec/tx';

import { ContractDatabase } from '../contract_database/index.js';
import { NoteDao } from './note_dao.js';
import { TxDao } from './tx_dao.js';

export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;

  addNote(note: NoteDao): Promise<void>;
  getNotes(contractAddress: AztecAddress, storageSlot: Fr): Promise<NoteDao[]>;
  addOrUpdateTx(tx: TxDao): Promise<void>;
}
