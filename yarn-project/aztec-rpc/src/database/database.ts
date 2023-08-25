import { CompleteAddress, HistoricBlockData } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractDatabase, MerkleTreeId, PublicKey } from '@aztec/types';

import { NoteSpendingInfoDao } from './note_spending_info_dao.js';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface Database extends ContractDatabase {
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
   * Adds complete address to the database.
   * @param address - The complete address to add.
   * @returns A promise resolving to true if the address was added, false if it already exists.
   * @throws If we try to add a CompleteAddress with the same AztecAddress but different public key or partial
   * address.
   */
  addCompleteAddress(address: CompleteAddress): Promise<boolean>;

  /**
   * Retrieves the complete address corresponding to the provided aztec address.
   * @param address - The aztec address of the complete address contract.
   * @returns A promise that resolves to a CompleteAddress instance if the address is found, or undefined if not found.
   */
  getCompleteAddress(address: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Retrieves the list of complete address added to this database
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  getCompleteAddresses(): Promise<CompleteAddress[]>;
}
