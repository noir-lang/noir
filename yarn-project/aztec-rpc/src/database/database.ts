import { PartialContractAddress } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { ContractDatabase, MerkleTreeId, PublicKey, TxHash } from '@aztec/types';

import { NoteSpendingInfoDao } from './note_spending_info_dao.js';
import { TxDao } from './tx_dao.js';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  getTxsByAddress(origin: AztecAddress): Promise<TxDao[]>;
  addTx(tx: TxDao): Promise<void>;
  addTxs(txs: TxDao[]): Promise<void>;

  getNoteSpendingInfo(contract: AztecAddress, storageSlot: Fr): Promise<NoteSpendingInfoDao[]>;
  addNoteSpendingInfo(noteSpendingInfoDao: NoteSpendingInfoDao): Promise<void>;
  addNoteSpendingInfoBatch(noteSpendingInfoDaos: NoteSpendingInfoDao[]): Promise<void>;
  removeNullifiedNoteSpendingInfo(nullifiers: Fr[], account: Point): Promise<NoteSpendingInfoDao[]>;

  getTreeRoots(): Record<MerkleTreeId, Fr>;
  setTreeRoots(roots: Record<MerkleTreeId, Fr>): Promise<void>;

  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialContractAddress,
  ): Promise<void>;
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[Point, PartialContractAddress] | undefined>;
  getAccounts(): Promise<AztecAddress[]>;
}
