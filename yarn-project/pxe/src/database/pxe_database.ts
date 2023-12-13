import { BlockHeader, CompleteAddress, PublicKey } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractDatabase, MerkleTreeId, NoteFilter } from '@aztec/types';

import { NoteDao } from './note_dao.js';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface PxeDatabase extends ContractDatabase {
  /**
   * Add a auth witness to the database.
   * @param messageHash - The message hash.
   * @param witness - An array of field elements representing the auth witness.
   */
  addAuthWitness(messageHash: Fr, witness: Fr[]): Promise<void>;

  /**
   * Fetching the auth witness for a given message hash.
   * @param messageHash - The message hash.
   * @returns A Promise that resolves to an array of field elements representing the auth witness.
   */
  getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined>;

  /**
   * Adding a capsule to the capsule dispenser.
   * @remarks A capsule is a "blob" of data that is passed to the contract through an oracle.
   * @param capsule - An array of field elements representing the capsule.
   */
  addCapsule(capsule: Fr[]): Promise<void>;

  /**
   * Get the next capsule from the capsule dispenser.
   * @remarks A capsule is a "blob" of data that is passed to the contract through an oracle.
   * @returns A promise that resolves to an array of field elements representing the capsule.
   */
  popCapsule(): Promise<Fr[] | undefined>;

  /**
   * Gets notes based on the provided filter.
   * @param filter - The filter to apply to the notes.
   * @returns The requested notes.
   */
  getNotes(filter: NoteFilter): Promise<NoteDao[]>;

  /**
   * Adds a note to DB.
   * @param note - The note to add.
   */
  addNote(note: NoteDao): Promise<void>;

  /**
   * Adds an array of notes to DB.
   * This function is used to insert multiple notes to the database at once,
   * which can improve performance when dealing with large numbers of transactions.
   *
   * @param notes - An array of notes.
   */
  addNotes(notes: NoteDao[]): Promise<void>;

  /**
   * Remove nullified notes associated with the given account and nullifiers.
   *
   * @param nullifiers - An array of Fr instances representing nullifiers to be matched.
   * @param account - A PublicKey instance representing the account for which the records are being removed.
   * @returns Removed notes.
   */
  removeNullifiedNotes(nullifiers: Fr[], account: PublicKey): Promise<NoteDao[]>;

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
   * Retrieve the stored Block Header from the database.
   * The function returns a Promise that resolves to the Block Header.
   * This data is required to reproduce block attestations.
   * Throws an error if the block header is not available within the database.
   *
   * note: this data is a combination of the tree roots and the global variables hash.
   */
  getBlockHeader(): BlockHeader;

  /**
   * Set the latest Block Header.
   * This function updates the 'global variables hash' and `tree roots` property of the instance
   * Note that this will overwrite any existing hash or roots in the database.
   *
   * @param blockHeader - An object containing the most recent block header.
   * @returns A Promise that resolves when the hash has been successfully updated in the database.
   */
  setBlockHeader(blockHeader: BlockHeader): Promise<void>;

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

  /**
   * Returns the estimated size in bytes of this db.
   * @returns The estimated size in bytes of this db.
   */
  estimateSize(): number;
}
