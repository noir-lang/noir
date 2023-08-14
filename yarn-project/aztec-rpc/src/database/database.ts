import { HistoricBlockData, PartialAddress } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractDatabase, MerkleTreeId, PublicKey, TxHash } from '@aztec/types';

import { NoteSpendingInfoDao } from './note_spending_info_dao.js';
import { TxDao } from './tx_dao.js';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface Database extends ContractDatabase {
  /**
   * Retrieve a transaction from the MemoryDB using its transaction hash.
   * The function searches for the transaction with the given hash in the txTable and returns it as a Promise.
   * Returns 'undefined' if the transaction is not found in the database.
   *
   * @param txHash - The TxHash of the transaction to be retrieved.
   * @returns A Promise that resolves to the found TxDao instance, or undefined if not found.
   */
  getTx(txHash: TxHash): Promise<TxDao | undefined>;

  /**
   * Adds a TxDao instance to the transaction table.
   * If a transaction with the same hash already exists in the table, it replaces the existing one.
   * Otherwise, it pushes the new transaction to the table.
   *
   * @param tx - The TxDao instance representing the transaction to be added.
   * @returns A Promise that resolves when the transaction is successfully added/updated in the table.
   */
  addTx(tx: TxDao): Promise<void>;

  /**
   * Add an array of transaction data objects.
   * If a transaction with the same hash already exists in the database, it will be updated
   * with the new transaction data. Otherwise, the new transaction will be added to the database.
   *
   * @param txs - An array of TxDao instances representing the transactions to be added to the database.
   * @returns A Promise that resolves when all the transactions have been added or updated.
   */
  addTxs(txs: TxDao[]): Promise<void>;

  /**
   * Get auxiliary transaction data based on contract address and storage slot.
   * It searches for matching NoteSpendingInfoDao objects in the MemoryDB's noteSpendingInfoTable
   * where both the contractAddress and storageSlot properties match the given inputs.
   *
   * @param contract - The contract address.
   * @param storageSlot - A Fr object representing the storage slot to search for in the auxiliary data.
   * @returns An array of NoteSpendingInfoDao objects that fulfill the contract address and storage slot criteria.
   */
  getNoteSpendingInfo(contract: AztecAddress, storageSlot: Fr): Promise<NoteSpendingInfoDao[]>;

  /**
   * Add a NoteSpendingInfoDao instance to the noteSpendingInfoTable.
   * This function is used to store auxiliary data related to a transaction,
   * such as contract address and storage slot, in the database.
   *
   * @param noteSpendingInfoDao - The NoteSpendingInfoDao instance containing the auxiliary data of a transaction.
   * @returns A promise that resolves when the auxiliary data is added to the database.
   */
  addNoteSpendingInfo(noteSpendingInfoDao: NoteSpendingInfoDao): Promise<void>;

  /**
   * Adds an array of NoteSpendingInfoDaos to the noteSpendingInfoTable.
   * This function is used to insert multiple transaction auxiliary data objects into the database at once,
   * which can improve performance when dealing with large numbers of transactions.
   *
   * @param noteSpendingInfoDaos - An array of NoteSpendingInfoDao instances representing the auxiliary data of transactions.
   * @returns A Promise that resolves when all NoteSpendingInfoDaos have been successfully added to the noteSpendingInfoTable.
   */
  addNoteSpendingInfoBatch(noteSpendingInfoDaos: NoteSpendingInfoDao[]): Promise<void>;

  /**
   * Remove nullified transaction auxiliary data records associated with the given account and nullifiers.
   * The function filters the records based on matching account and nullifier values, and updates the
   * noteSpendingInfoTable with the remaining records. It returns an array of removed NoteSpendingInfoDao instances.
   *
   * @param nullifiers - An array of Fr instances representing nullifiers to be matched.
   * @param account - A PublicKey instance representing the account for which the records are being removed.
   * @returns A Promise resolved with an array of removed NoteSpendingInfoDao instances.
   */
  removeNullifiedNoteSpendingInfo(nullifiers: Fr[], account: PublicKey): Promise<NoteSpendingInfoDao[]>;

  /**
   * Retrieve the stored Merkle tree roots from the database.
   * The function returns a Promise that resolves to an object containing the MerkleTreeId as keys
   * and their corresponding Fr values as roots. Throws an error if the tree roots are not set in the
   * memory database.
   *
   * @returns An object containing the Merkle tree roots for each merkle tree id.
   */
  getTreeRoots(): Record<MerkleTreeId, Fr>;

  /**
   * Set the tree roots for the Merkle trees in the database.
   * This function updates the 'treeRoots' property of the instance
   * with the provided 'roots' object containing MerkleTreeId and Fr pairs.
   * Note that this will overwrite any existing tree roots in the database.
   *
   * @param roots - A Record object mapping MerkleTreeIds to their corresponding Fr root values.
   * @returns A Promise that resolves when the tree roots have been successfully updated in the database.
   */
  setTreeRoots(roots: Record<MerkleTreeId, Fr>): Promise<void>;

  /**
   * Retrieve the stored Historic Block Data from the database.
   * The function returns a Promise that resolves to the Historic Block Data.
   * This data is required to reproduce block attestations.
   * Throws an error if the historic block data is not available within the database.
   *
   * note: this data is a combination of the tree roots and the global variables hash.
   */
  getHistoricBlockData(): HistoricBlockData;

  /**
   * Set the latest Historic Block Data.
   * This function updates the 'global variables hash' and `tree roots` property of the instance
   * Note that this will overwrite any existing hash or roots in the database.
   *
   * @param historicBlockData - An object containing the most recent historic block data.
   * @returns A Promise that resolves when the hash has been successfully updated in the database.
   */
  setHistoricBlockData(historicBlockData: HistoricBlockData): Promise<void>;

  /**
   * Adds public key and partial address to a database.
   * @param address - Address of the account to add public key and partial address for.
   * @param publicKey - Public key of the corresponding user.
   * @param partialAddress - The partially computed address of the account contract.
   * @returns A Promise that resolves once the public key has been added to the database.
   */
  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialAddress,
  ): Promise<void>;

  /**
   * Retrieve the public key and partial address associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key and partial address for.
   * @returns A Promise resolving to the PublicKey instance representing the public key.
   * @remarks The public key and partial address form a preimage of a contract address. See
   * https://github.com/AztecProtocol/aztec-packages/blob/janb/rpc-interface-cleanup/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
   */
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, PartialAddress] | undefined>;

  /**
   * Retrieves the list of Aztec addresses added to this database
   * The addresses are returned as a promise that resolves to an array of AztecAddress objects.
   *
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  getAccounts(): Promise<AztecAddress[]>;
}
