import { type NoteFilter } from '@aztec/circuit-types';
import { type CompleteAddress, type Header, type PublicKey } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type ContractArtifactDatabase } from './contracts/contract_artifact_db.js';
import { type ContractInstanceDatabase } from './contracts/contract_instance_db.js';
import { type DeferredNoteDao } from './deferred_note_dao.js';
import { type IncomingNoteDao } from './incoming_note_dao.js';
import { type OutgoingNoteDao } from './outgoing_note_dao.js';

/**
 * A database interface that provides methods for retrieving, adding, and removing transactional data related to Aztec
 * addresses, storage slots, and nullifiers.
 */
export interface PxeDatabase extends ContractArtifactDatabase, ContractInstanceDatabase {
  getContract(address: AztecAddress): Promise<(ContractInstanceWithAddress & ContractArtifact) | undefined>;

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
  getNotes(filter: NoteFilter): Promise<IncomingNoteDao[]>;

  /**
   * Adds a note to DB.
   * @param note - The note to add.
   */
  addNote(note: IncomingNoteDao): Promise<void>;

  /**
   * Adds an array of notes to DB.
   * This function is used to insert multiple notes to the database at once,
   * which can improve performance when dealing with large numbers of transactions.
   *
   * @param incomingNotes - An array of notes which were decrypted as incoming.
   * @param outgoingNotes - An array of notes which were decrypted as outgoing.
   */
  addNotes(incomingNotes: IncomingNoteDao[], outgoingNotes: OutgoingNoteDao[]): Promise<void>;

  /**
   * Add notes to the database that are intended for us, but we don't yet have the contract.
   * @param deferredNotes - An array of deferred notes.
   */
  addDeferredNotes(deferredNotes: DeferredNoteDao[]): Promise<void>;

  /**
   * Get deferred notes for a given contract address.
   * @param contractAddress - The contract address to get the deferred notes for.
   */
  getDeferredNotesByContract(contractAddress: AztecAddress): Promise<DeferredNoteDao[]>;

  /**
   * Remove deferred notes for a given contract address.
   * @param contractAddress - The contract address to remove the deferred notes for.
   * @returns an array of the removed deferred notes
   */
  removeDeferredNotesByContract(contractAddress: AztecAddress): Promise<DeferredNoteDao[]>;

  /**
   * Remove nullified notes associated with the given account and nullifiers.
   *
   * @param nullifiers - An array of Fr instances representing nullifiers to be matched.
   * @param account - A PublicKey instance representing the account for which the records are being removed.
   * @returns Removed notes.
   */
  removeNullifiedNotes(nullifiers: Fr[], account: PublicKey): Promise<IncomingNoteDao[]>;

  /**
   * Gets the most recently processed block number.
   * @returns The most recently processed block number or undefined if never synched.
   */
  getBlockNumber(): number | undefined;

  /**
   * Retrieve the stored Block Header from the database.
   * The function returns a Promise that resolves to the Block Header.
   * This data is required to reproduce block attestations.
   * Throws an error if the block header is not available within the database.
   *
   * note: this data is a combination of the tree roots and the global variables hash.
   *
   * @returns The Block Header.
   * @throws If no block have been processed yet.
   */
  getHeader(): Header;

  /**
   * Set the latest Block Header.
   * Note that this will overwrite any existing hash or roots in the database.
   *
   * @param header - An object containing the most recent block header.
   * @returns A Promise that resolves when the hash has been successfully updated in the database.
   */
  setHeader(header: Header): Promise<void>;

  /**
   * Adds complete address to the database.
   * @param address - The complete address to add.
   * @returns A promise resolving to true if the address was added, false if it already exists.
   * @throws If we try to add a CompleteAddress with the same AztecAddress but different public key or partial
   * address.
   */
  addCompleteAddress(address: CompleteAddress): Promise<boolean>;

  /**
   * Retrieve the complete address associated to a given address.
   * @param account - The account address.
   * @returns A promise that resolves to a CompleteAddress instance if found, or undefined if not found.
   */
  getCompleteAddress(account: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Retrieves the list of complete addresses added to this database
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  getCompleteAddresses(): Promise<CompleteAddress[]>;

  /**
   * Updates up to which block number we have processed notes for a given public key.
   * @param publicKey - The public key to set the synched block number for.
   * @param blockNumber - The block number to set.
   */
  setSynchedBlockNumberForPublicKey(publicKey: PublicKey, blockNumber: number): Promise<void>;

  /**
   * Get the synched block number for a given public key.
   * @param publicKey - The public key to get the synched block number for.
   */
  getSynchedBlockNumberForPublicKey(publicKey: PublicKey): number | undefined;

  /**
   * Returns the estimated size in bytes of this db.
   * @returns The estimated size in bytes of this db.
   */
  estimateSize(): number;
}
