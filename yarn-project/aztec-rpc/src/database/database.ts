import { ContractDatabase, TxHash, PublicKey } from '@aztec/types';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { MerkleTreeId } from '@aztec/types';
import { PartialContractAddress } from '@aztec/circuits.js';

import { NoteSpendingInfoDao } from './note_spending_info_dao.js';
import { TxDao } from './tx_dao.js';

/**
 * Options for selecting items from the database.
 */
export interface GetOptions {
  /**
   * An array of indices of the fields to sort.
   * Default: empty array.
   */
  sortBy?: number[];
  /**
   * The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * Default: empty array.
   */
  sortOrder?: number[];
  /**
   * The number of items to retrieve per query.
   * Default: 0. No limit.
   */
  limit?: number;
  /**
   * The starting index for pagination.
   * Default: 0.
   */
  offset?: number;
}

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  getTxsByAddress(from: AztecAddress): Promise<TxDao[]>;
  addTx(tx: TxDao): Promise<void>;
  addTxs(txs: TxDao[]): Promise<void>;

  getNoteSpendingInfo(contract: AztecAddress, storageSlot: Fr, options?: GetOptions): Promise<NoteSpendingInfoDao[]>;
  addNoteSpendingInfo(noteSpendingInfoDao: NoteSpendingInfoDao): Promise<void>;
  addNoteSpendingInfoBatch(noteSpendingInfoDaos: NoteSpendingInfoDao[]): Promise<void>;
  removeNullifiedNoteSpendingInfo(nullifiers: Fr[], account: Point): Promise<NoteSpendingInfoDao[]>;

  getTreeRoots(): Record<MerkleTreeId, Fr>;
  setTreeRoots(roots: Record<MerkleTreeId, Fr>): Promise<void>;

  addPublicKey(address: AztecAddress, publicKey: PublicKey, partialAddress: PartialContractAddress): Promise<void>;
  getPublicKey(address: AztecAddress): Promise<[Point, PartialContractAddress] | undefined>;
}
