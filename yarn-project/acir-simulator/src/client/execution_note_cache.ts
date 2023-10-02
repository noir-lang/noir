import { CircuitsWasm, EMPTY_NULLIFIED_COMMITMENT } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { NoteData } from '../acvm/index.js';

/**
 * Data that's accessible by all the function calls in an execution.
 */
export class ExecutionNoteCache {
  /**
   * New notes created in this transaction.
   * This mapping maps from a contract address to the notes in the contract.
   */
  private newNotes: Map<bigint, NoteData[]> = new Map();

  /**
   * The list of nullifiers created in this transaction.
   * This mapping maps from a contract address to the nullifiers emitted from the contract.
   * The note which is nullified might be new or not (i.e., was generated in a previous transaction).
   * Note that their value (bigint representation) is used because Frs cannot be looked up in Sets.
   */
  private nullifiers: Map<bigint, Set<bigint>> = new Map();

  /**
   * Add a new note to cache.
   * @param note - New note created during execution.
   */
  public addNewNote(note: NoteData) {
    const notes = this.newNotes.get(note.contractAddress.toBigInt()) ?? [];
    notes.push(note);
    this.newNotes.set(note.contractAddress.toBigInt(), notes);
  }

  /**
   * Add a nullifier to cache. It could be for a db note or a new note created during execution.
   * @param contractAddress - Contract address of the note.
   * @param storageSlot - Storage slot of the note.
   * @param innerNullifier - Inner nullifier of the note.
   * @param innerNoteHash - Inner note hash of the note. If this value equals EMPTY_NULLIFIED_COMMITMENT, it means the
   * note being nullified is from a previous transaction (and thus not a new note).
   */
  public async nullifyNote(contractAddress: AztecAddress, innerNullifier: Fr, innerNoteHash: Fr) {
    const wasm = await CircuitsWasm.get();
    const siloedNullifier = siloNullifier(wasm, contractAddress, innerNullifier);
    const nullifiers = this.getNullifiers(contractAddress);
    nullifiers.add(siloedNullifier.value);
    this.nullifiers.set(contractAddress.toBigInt(), nullifiers);

    // Find and remove the matching new note if the emitted innerNoteHash is not empty.
    if (!innerNoteHash.equals(new Fr(EMPTY_NULLIFIED_COMMITMENT))) {
      const notes = this.newNotes.get(contractAddress.toBigInt()) ?? [];
      const noteIndexToRemove = notes.findIndex(n => n.innerNoteHash.equals(innerNoteHash));
      if (noteIndexToRemove === -1) {
        throw new Error('Attempt to remove a pending note that does not exist.');
      }
      notes.splice(noteIndexToRemove, 1);
      this.newNotes.set(contractAddress.toBigInt(), notes);
    }
  }

  /**
   * Return notes created up to current point in execution.
   * If a nullifier for a note in this list is emitted, the note will be deleted.
   * @param contractAddress - Contract address of the notes.
   * @param storageSlot - Storage slot of the notes.
   **/
  public getNotes(contractAddress: AztecAddress, storageSlot: Fr) {
    const notes = this.newNotes.get(contractAddress.toBigInt()) ?? [];
    return notes.filter(n => n.storageSlot.equals(storageSlot));
  }

  /**
   * Check if a note exists in the newNotes array.
   * @param contractAddress - Contract address of the note.
   * @param storageSlot - Storage slot of the note.
   * @param innerNoteHash - Inner note hash of the note.
   **/
  public checkNoteExists(contractAddress: AztecAddress, innerNoteHash: Fr) {
    const notes = this.newNotes.get(contractAddress.toBigInt()) ?? [];
    return notes.some(n => n.innerNoteHash.equals(innerNoteHash));
  }

  /**
   * Return all nullifiers emitted from a contract.
   * @param contractAddress - Address of the contract.
   */
  public getNullifiers(contractAddress: AztecAddress): Set<bigint> {
    return this.nullifiers.get(contractAddress.toBigInt()) ?? new Set();
  }
}
