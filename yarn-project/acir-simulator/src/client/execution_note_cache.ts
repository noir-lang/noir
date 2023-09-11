import { CircuitsWasm, EMPTY_NULLIFIED_COMMITMENT } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { NoteData } from './db_oracle.js';

/**
 * Generate a key with the given contract address and storage slot.
 * @param contractAddress - Contract address.
 * @param storageSlot - Storage slot.
 */
const generateKeyForContractStorageSlot = (contractAddress: AztecAddress, storageSlot: Fr) =>
  `${contractAddress.toShortString}${storageSlot}`;

/**
 * Data that's accessible by all the function calls in an execution.
 */
export class ExecutionNoteCache {
  /**
   * Notes created during execution.
   * The key of the mapping is a value representing a contract address and storage slot combination.
   */
  private newNotes: Map<string, NoteData[]> = new Map();

  /**
   * The list of nullifiers created in this transaction.
   * The key of the mapping is a value representing a contract address and storage slot combination.
   * The note which is nullified might be new or not (i.e., was generated in a previous transaction).
   * Note that their value (bigint representation) is used because Frs cannot be looked up in Sets.
   */
  private nullifiers: Map<string, Set<bigint>> = new Map();

  /**
   * Add a new note to cache.
   * @param note - New note created during execution.
   */
  public addNewNote(note: NoteData) {
    const key = generateKeyForContractStorageSlot(note.contractAddress, note.storageSlot);
    const notes = this.newNotes.get(key) ?? [];
    notes.push(note);
    this.newNotes.set(key, notes);
  }

  /**
   * Add a nullifier to cache. It could be for a db note or a new note created during execution.
   * @param contractAddress - Contract address of the note.
   * @param storageSlot - Storage slot of the note.
   * @param innerNullifier - Inner nullifier of the note.
   * @param innerNoteHash - Inner note hash of the note. If this value equals EMPTY_NULLIFIED_COMMITMENT, it means the
   * note being nullified is from a previous transaction (and thus not a new note).
   */
  public async nullifyNote(contractAddress: AztecAddress, storageSlot: Fr, innerNullifier: Fr, innerNoteHash: Fr) {
    const wasm = await CircuitsWasm.get();
    const siloedNullifier = siloNullifier(wasm, contractAddress, innerNullifier);
    const nullifiers = this.getNullifiers(contractAddress, storageSlot);
    if (nullifiers.has(siloedNullifier.value)) {
      throw new Error('Attemp to nullify the same note twice.');
    }

    nullifiers.add(siloedNullifier.value);
    const key = generateKeyForContractStorageSlot(contractAddress, storageSlot);
    this.nullifiers.set(key, nullifiers);

    // Find and remove the matching new note if the emitted innerNoteHash is not empty.
    if (!innerNoteHash.equals(new Fr(EMPTY_NULLIFIED_COMMITMENT))) {
      const notes = this.newNotes.get(key) ?? [];
      /**
       * The identifier used to determine matching is the inner note hash value.
       * However, we adopt a defensive approach and ensure that the contract address
       * and storage slot do match.
       */
      const noteIndexToRemove = notes.findIndex(n => n.innerNoteHash.equals(innerNoteHash));
      if (noteIndexToRemove === -1) {
        throw new Error('Attemp to remove a pending note that does not exist.');
      }
      notes.splice(noteIndexToRemove, 1);
      this.newNotes.set(key, notes);
    }
  }

  /**
   * Return notes created up to current point in execution.
   * If a nullifier for a note in this list is emitted, the note will be deleted.
   * @param contractAddress - Contract address of the notes.
   * @param storageSlot - Storage slot of the notes.
   **/
  public getNotes(contractAddress: AztecAddress, storageSlot: Fr) {
    const key = generateKeyForContractStorageSlot(contractAddress, storageSlot);
    return this.newNotes.get(key) ?? [];
  }

  /**
   * Return all nullifiers of a storage slot.
   * @param contractAddress - Contract address of the notes.
   * @param storageSlot - Storage slot of the notes.
   */
  public getNullifiers(contractAddress: AztecAddress, storageSlot: Fr): Set<bigint> {
    const key = generateKeyForContractStorageSlot(contractAddress, storageSlot);
    return this.nullifiers.get(key) || new Set();
  }
}
