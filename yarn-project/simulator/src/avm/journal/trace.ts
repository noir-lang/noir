import { Fr } from '@aztec/foundation/fields';

import {
  type TracedL1toL2MessageCheck,
  type TracedNoteHash,
  type TracedNoteHashCheck,
  type TracedNullifier,
  type TracedNullifierCheck,
  type TracedPublicStorageRead,
  type TracedPublicStorageWrite,
} from './trace_types.js';

export class WorldStateAccessTrace {
  public accessCounter: number;

  public publicStorageReads: TracedPublicStorageRead[] = [];
  public publicStorageWrites: TracedPublicStorageWrite[] = [];

  public noteHashChecks: TracedNoteHashCheck[] = [];
  public newNoteHashes: TracedNoteHash[] = [];
  public nullifierChecks: TracedNullifierCheck[] = [];
  public newNullifiers: TracedNullifier[] = [];
  public l1ToL2MessageChecks: TracedL1toL2MessageCheck[] = [];

  //public contractCalls: TracedContractCall[] = [];
  //public archiveChecks: TracedArchiveLeafCheck[] = [];

  constructor(parentTrace?: WorldStateAccessTrace) {
    this.accessCounter = parentTrace ? parentTrace.accessCounter : 0;
  }

  public getAccessCounter() {
    return this.accessCounter;
  }

  public tracePublicStorageRead(storageAddress: Fr, slot: Fr, value: Fr, exists: boolean) {
    // TODO(4805): check if some threshold is reached for max storage reads
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    const traced: TracedPublicStorageRead = {
      //  callPointer: Fr.ZERO,
      storageAddress,
      slot,
      value,
      exists,
      counter: new Fr(this.accessCounter),
      //  endLifetime: Fr.ZERO,
    };
    this.publicStorageReads.push(traced);
    this.incrementAccessCounter();
  }

  public tracePublicStorageWrite(storageAddress: Fr, slot: Fr, value: Fr) {
    // TODO(4805): check if some threshold is reached for max storage writes
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    const traced: TracedPublicStorageWrite = {
      //  callPointer: Fr.ZERO,
      storageAddress,
      slot,
      value,
      counter: new Fr(this.accessCounter),
      //  endLifetime: Fr.ZERO,
    };
    this.publicStorageWrites.push(traced);
    this.incrementAccessCounter();
  }

  public traceNoteHashCheck(storageAddress: Fr, noteHash: Fr, exists: boolean, leafIndex: Fr) {
    const traced: TracedNoteHashCheck = {
      // callPointer: Fr.ZERO,
      storageAddress,
      noteHash,
      exists,
      counter: new Fr(this.accessCounter),
      // endLifetime: Fr.ZERO,
      leafIndex,
    };
    this.noteHashChecks.push(traced);
    this.incrementAccessCounter();
  }

  public traceNewNoteHash(storageAddress: Fr, noteHash: Fr) {
    // TODO(4805): check if some threshold is reached for max new note hash
    const traced: TracedNoteHash = {
      //  callPointer: Fr.ZERO,
      storageAddress,
      noteHash,
      counter: new Fr(this.accessCounter),
      //  endLifetime: Fr.ZERO,
    };
    this.newNoteHashes.push(traced);
    this.incrementAccessCounter();
  }

  public traceNullifierCheck(storageAddress: Fr, nullifier: Fr, exists: boolean, isPending: boolean, leafIndex: Fr) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    const traced: TracedNullifierCheck = {
      // callPointer: Fr.ZERO,
      storageAddress,
      nullifier,
      exists,
      counter: new Fr(this.accessCounter),
      // endLifetime: Fr.ZERO,
      isPending,
      leafIndex,
    };
    this.nullifierChecks.push(traced);
    this.incrementAccessCounter();
  }

  public traceNewNullifier(storageAddress: Fr, nullifier: Fr) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    const tracedNullifier: TracedNullifier = {
      // callPointer: Fr.ZERO,
      storageAddress,
      nullifier,
      counter: new Fr(this.accessCounter),
      // endLifetime: Fr.ZERO,
    };
    this.newNullifiers.push(tracedNullifier);
    this.incrementAccessCounter();
  }

  public traceL1ToL2MessageCheck(msgHash: Fr, msgLeafIndex: Fr, exists: boolean) {
    // TODO(4805): check if some threshold is reached for max message reads
    const traced: TracedL1toL2MessageCheck = {
      //callPointer: Fr.ZERO, // FIXME
      leafIndex: msgLeafIndex,
      msgHash: msgHash,
      exists: exists,
      //endLifetime: Fr.ZERO, // FIXME
    };
    this.l1ToL2MessageChecks.push(traced);
    this.incrementAccessCounter();
  }

  private incrementAccessCounter() {
    this.accessCounter++;
  }

  /**
   * Merges another trace into this one
   *
   * - Public state journals (r/w logs), with the accessing being appended in chronological order
   * - Utxo objects are concatenated
   *
   * @param incomingTrace - the incoming trace to merge into this instance
   */
  public acceptAndMerge(incomingTrace: WorldStateAccessTrace) {
    // Merge storage read and write journals
    this.publicStorageReads = this.publicStorageReads.concat(incomingTrace.publicStorageReads);
    this.publicStorageWrites = this.publicStorageWrites.concat(incomingTrace.publicStorageWrites);
    // Merge new note hashes and nullifiers
    this.noteHashChecks = this.noteHashChecks.concat(incomingTrace.noteHashChecks);
    this.newNoteHashes = this.newNoteHashes.concat(incomingTrace.newNoteHashes);
    this.nullifierChecks = this.nullifierChecks.concat(incomingTrace.nullifierChecks);
    this.newNullifiers = this.newNullifiers.concat(incomingTrace.newNullifiers);
    this.l1ToL2MessageChecks = this.l1ToL2MessageChecks.concat(incomingTrace.l1ToL2MessageChecks);
    // it is assumed that the incoming trace was initialized with this as parent, so accept counter
    this.accessCounter = incomingTrace.accessCounter;
  }
}
