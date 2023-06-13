import { TxHash } from '@aztec/types';
import { ContractDatabase } from '../contract_database/index.js';
import { NoteSpendingInfoDao } from './note_spending_info_dao.js';
import { TxDao } from './tx_dao.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { MerkleTreeId } from '@aztec/types';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  getTxsByAddress(from: AztecAddress): Promise<TxDao[]>;
  addTx(tx: TxDao): Promise<void>;
  addTxs(txs: TxDao[]): Promise<void>;

  getNoteSpendingInfo(contract: AztecAddress, storageSlot: Fr): Promise<NoteSpendingInfoDao[]>;
  addNoteSpendingInfo(noteSpendingInfoDao: NoteSpendingInfoDao): Promise<void>;
  addNoteSpendingInfoBatch(noteSpendingInfoDaos: NoteSpendingInfoDao[]): Promise<void>;
  removeNullifiedNoteSpendingInfo(nullifiers: Fr[], account: Point): Promise<NoteSpendingInfoDao[]>;

  getTreeRoots(): Record<MerkleTreeId, Fr>;
  setTreeRoots(roots: Record<MerkleTreeId, Fr>): Promise<void>;
}
