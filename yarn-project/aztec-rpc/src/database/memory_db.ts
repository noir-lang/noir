import { TxHash } from '@aztec/types';
import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { TxAuxDataDao } from './tx_aux_data_dao.js';
import { TxDao } from './tx_dao.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { MerkleTreeId } from '@aztec/types';

/**
 * The MemoryDB class provides an in-memory implementation of a database to manage transactions and auxiliary data.
 * It extends the MemoryContractDatabase, allowing it to store contract-related data as well.
 * The class offers methods to add, fetch, and remove transaction records and auxiliary data based on various filters such as transaction hash, address, and storage slot.
 * As an in-memory database, the stored data will not persist beyond the life of the application instance.
 */
export class MemoryDB extends MemoryContractDatabase implements Database {
  private txTable: TxDao[] = [];
  private txAuxDataTable: TxAuxDataDao[] = [];
  private treeRoots: Record<MerkleTreeId, Fr> | undefined;

  /**
   * Retrieve a transaction from the MemoryDB using its transaction hash.
   * The function searches for the transaction with the given hash in the txTable and returns it as a Promise.
   * Returns 'undefined' if the transaction is not found in the database.
   *
   * @param txHash - The TxHash of the transaction to be retrieved.
   * @returns A Promise that resolves to the found TxDao instance, or undefined if not found.
   */
  public getTx(txHash: TxHash) {
    return Promise.resolve(this.txTable.find(tx => tx.txHash.equals(txHash)));
  }

  /**
   * Retrieve all transactions associated with a given AztecAddress.
   *
   * @param from - The sender's address.
   * @returns A Promise resolving to an array of TxDao objects associated with the sender.
   */
  public getTxsByAddress(from: AztecAddress) {
    return Promise.resolve(this.txTable.filter(tx => tx.from.equals(from)));
  }

  /**
   * Adds a TxDao instance to the transaction table.
   * If a transaction with the same hash already exists in the table, it replaces the existing one.
   * Otherwise, it pushes the new transaction to the table.
   *
   * @param tx - The TxDao instance representing the transaction to be added.
   * @returns A Promise that resolves when the transaction is successfully added/updated in the table.
   */
  public addTx(tx: TxDao) {
    const index = this.txTable.findIndex(t => t.txHash.equals(tx.txHash));
    if (index === -1) {
      this.txTable.push(tx);
    } else {
      this.txTable[index] = tx;
    }
    return Promise.resolve();
  }

  /**
   * Add an array of transaction data objects.
   * If a transaction with the same hash already exists in the database, it will be updated
   * with the new transaction data. Otherwise, the new transaction will be added to the database.
   *
   * @param txs - An array of TxDao instances representing the transactions to be added to the database.
   * @returns A Promise that resolves when all the transactions have been added or updated.
   */
  public async addTxs(txs: TxDao[]) {
    await Promise.all(txs.map(tx => this.addTx(tx)));
  }

  /**
   * Add a TxAuxDataDao instance to the txAuxDataTable.
   * This function is used to store auxiliary data related to a transaction,
   * such as contract address and storage slot, in the database.
   *
   * @param txAuxDataDao - The TxAuxDataDao instance containing the auxiliary data of a transaction.
   * @returns A promise that resolves when the auxiliary data is added to the database.
   */
  public addTxAuxData(txAuxDataDao: TxAuxDataDao) {
    this.txAuxDataTable.push(txAuxDataDao);
    return Promise.resolve();
  }

  /**
   * Adds an array of TxAuxDataDaos to the txAuxDataTable.
   * This function is used to insert multiple transaction auxiliary data objects into the database at once,
   * which can improve performance when dealing with large numbers of transactions.
   *
   * @param txAuxDataDaos - An array of TxAuxDataDao instances representing the auxiliary data of transactions.
   * @returns A Promise that resolves when all TxAuxDataDaos have been successfully added to the txAuxDataTable.
   */
  public addTxAuxDataBatch(txAuxDataDaos: TxAuxDataDao[]) {
    this.txAuxDataTable.push(...txAuxDataDaos);
    return Promise.resolve();
  }

  /**
   * Get auxiliary transaction data based on contract address and storage slot.
   * It searches for matching TxAuxDataDao objects in the MemoryDB's txAuxDataTable
   * where both the contractAddress and storageSlot properties match the given inputs.
   *
   * @param contract - The contract address.
   * @param storageSlot - A Fr object representing the storage slot to search for in the auxiliary data.
   * @returns An array of TxAuxDataDao objects that fulfill the contract address and storage slot criteria.
   */
  public getTxAuxData(contract: AztecAddress, storageSlot: Fr) {
    const res = this.txAuxDataTable.filter(
      txAuxData =>
        txAuxData.contractAddress.equals(contract) && txAuxData.storageSlot.toBuffer().equals(storageSlot.toBuffer()),
    );
    return Promise.resolve(res);
  }

  /**
   * Remove nullified transaction auxiliary data records associated with the given account and nullifiers.
   * The function filters the records based on matching account and nullifier values, and updates the
   * txAuxDataTable with the remaining records. It returns an array of removed TxAuxDataDao instances.
   *
   * @param nullifiers - An array of Fr instances representing nullifiers to be matched.
   * @param account - A Point instance representing the account for which the records are being removed.
   * @returns A Promise resolved with an array of removed TxAuxDataDao instances.
   */
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

  /**
   * Retrieve the stored Merkle tree roots from the database.
   * The function returns a Promise that resolves to an object containing the MerkleTreeId as keys
   * and their corresponding Fr values as roots. Throws an error if the tree roots are not set in the
   * memory database.
   *
   * @returns An object containing the Merkle tree roots for each merkle tree id.
   */
  public getTreeRoots(): Record<MerkleTreeId, Fr> {
    const roots = this.treeRoots;
    if (!roots) throw new Error(`Tree roots not set in memory database`);
    return roots;
  }

  /**
   * Set the tree roots for the Merkle trees in the database.
   * This function updates the 'treeRoots' property of the instance
   * with the provided 'roots' object containing MerkleTreeId and Fr pairs.
   * Note that this will overwrite any existing tree roots in the database.
   *
   * @param roots - A Record object mapping MerkleTreeIds to their corresponding Fr root values.
   * @returns A Promise that resolves when the tree roots have been successfully updated in the database.
   */
  public setTreeRoots(roots: Record<MerkleTreeId, Fr>) {
    this.treeRoots = roots;
    return Promise.resolve();
  }
}
