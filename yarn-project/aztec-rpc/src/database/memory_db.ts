import { PartialAddress } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { MerkleTreeId, PublicKey, TxHash } from '@aztec/types';

import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { NoteSpendingInfoDao } from './note_spending_info_dao.js';
import { TxDao } from './tx_dao.js';

/**
 * The MemoryDB class provides an in-memory implementation of a database to manage transactions and auxiliary data.
 * It extends the MemoryContractDatabase, allowing it to store contract-related data as well.
 * The class offers methods to add, fetch, and remove transaction records and auxiliary data based on various filters such as transaction hash, address, and storage slot.
 * As an in-memory database, the stored data will not persist beyond the life of the application instance.
 */
export class MemoryDB extends MemoryContractDatabase implements Database {
  private txTable: TxDao[] = [];
  private noteSpendingInfoTable: NoteSpendingInfoDao[] = [];
  private treeRoots: Record<MerkleTreeId, Fr> | undefined;
  private publicKeysAndPartialAddresses: Map<bigint, [PublicKey, PartialAddress]> = new Map();

  constructor(logSuffix?: string) {
    super(createDebugLogger(logSuffix ? 'aztec:memory_db_' + logSuffix : 'aztec:memory_db'));
  }

  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txTable.find(tx => tx.txHash.equals(txHash)));
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

  public addNoteSpendingInfo(noteSpendingInfoDao: NoteSpendingInfoDao) {
    this.noteSpendingInfoTable.push(noteSpendingInfoDao);
    return Promise.resolve();
  }

  public addNoteSpendingInfoBatch(noteSpendingInfoDaos: NoteSpendingInfoDao[]) {
    this.noteSpendingInfoTable.push(...noteSpendingInfoDaos);
    return Promise.resolve();
  }

  public getNoteSpendingInfo(contract: AztecAddress, storageSlot: Fr) {
    const res = this.noteSpendingInfoTable.filter(
      noteSpendingInfo =>
        noteSpendingInfo.contractAddress.equals(contract) &&
        noteSpendingInfo.storageSlot.toBuffer().equals(storageSlot.toBuffer()),
    );
    return Promise.resolve(res);
  }

  public removeNullifiedNoteSpendingInfo(nullifiers: Fr[], account: PublicKey) {
    const nullifierSet = new Set(nullifiers.map(nullifier => nullifier.toString()));
    const [remaining, removed] = this.noteSpendingInfoTable.reduce(
      (acc: [NoteSpendingInfoDao[], NoteSpendingInfoDao[]], noteSpendingInfo) => {
        const nullifier = noteSpendingInfo.siloedNullifier.toString();
        if (noteSpendingInfo.publicKey.equals(account) && nullifierSet.has(nullifier)) {
          acc[1].push(noteSpendingInfo);
        } else {
          acc[0].push(noteSpendingInfo);
        }
        return acc;
      },
      [[], []],
    );

    this.noteSpendingInfoTable = remaining;

    return Promise.resolve(removed);
  }

  public getTreeRoots(): Record<MerkleTreeId, Fr> {
    const roots = this.treeRoots;
    if (!roots) throw new Error(`Tree roots not set in memory database`);
    return roots;
  }

  public setTreeRoots(roots: Record<MerkleTreeId, Fr>) {
    this.treeRoots = roots;
    return Promise.resolve();
  }

  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialAddress,
  ): Promise<void> {
    if (this.publicKeysAndPartialAddresses.has(address.toBigInt())) {
      throw new Error(`Account ${address} already exists`);
    }
    this.publicKeysAndPartialAddresses.set(address.toBigInt(), [publicKey, partialAddress]);
    return Promise.resolve();
  }

  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, Fr] | undefined> {
    return Promise.resolve(this.publicKeysAndPartialAddresses.get(address.toBigInt()));
  }

  getAccounts(): Promise<AztecAddress[]> {
    const addresses = Array.from(this.publicKeysAndPartialAddresses.keys());
    return Promise.resolve(addresses.map(AztecAddress.fromBigInt));
  }
}
