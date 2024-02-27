import { Fr } from '@aztec/foundation/fields';

import { TracedNullifierCheck } from './trace_types.js';

export class WorldStateAccessTrace {
  public accessCounter: number;
  //public contractCalls: Array<TracedContractCall> = [];

  //public publicStorageReads: Array<TracedPublicStorageRead> = [];
  public publicStorageReads: Map<bigint, Map<bigint, Fr[]>> = new Map();
  //public publicStorageWrites: Array<TracedPublicStorageWrite> = [];
  public publicStorageWrites: Map<bigint, Map<bigint, Fr[]>> = new Map();

  //public noteHashChecks: TracedNoteHashCheck[] = [];
  //public newNoteHashes: TracedNoteHash[] = [];
  public newNoteHashes: Fr[] = [];
  public nullifierChecks: TracedNullifierCheck[] = [];
  //public newNullifiers: TracedNullifier[] = [];
  public newNullifiers: Fr[] = [];
  //public l1toL2MessageReads: TracedL1toL2MessageRead[] = [];
  //public archiveChecks: TracedArchiveLeafCheck[] = [];

  constructor(parentTrace?: WorldStateAccessTrace) {
    this.accessCounter = parentTrace ? parentTrace.accessCounter : 0;
  }

  public getAccessCounter() {
    return this.accessCounter;
  }

  public tracePublicStorageRead(storageAddress: Fr, slot: Fr, value: Fr /*, _exists: boolean*/) {
    // TODO(4805): check if some threshold is reached for max storage reads
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    //const traced: TracedPublicStorageRead = {
    //  callPointer: Fr.ZERO,
    //  storageAddress,
    //  slot,
    //  value,
    //  exists,
    //  counter: new Fr(this.accessCounter),
    //  endLifetime: Fr.ZERO,
    //};
    //this.publicStorageReads.push(traced);
    this.journalRead(storageAddress, slot, value);
    this.incrementAccessCounter();
  }

  public tracePublicStorageWrite(storageAddress: Fr, slot: Fr, value: Fr) {
    // TODO(4805): check if some threshold is reached for max storage writes
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    //const traced: TracedPublicStorageWrite = {
    //  callPointer: Fr.ZERO,
    //  storageAddress,
    //  slot,
    //  value,
    //  counter: new Fr(this.accessCounter),
    //  endLifetime: Fr.ZERO,
    //};
    //this.publicStorageWrites.push(traced);
    this.journalWrite(storageAddress, slot, value);
    this.incrementAccessCounter();
  }

  public traceNewNoteHash(_storageAddress: Fr, noteHash: Fr) {
    // TODO(4805): check if some threshold is reached for max new note hash
    //const traced: TracedNoteHash = {
    //  callPointer: Fr.ZERO,
    //  storageAddress,
    //  noteHash,
    //  counter: new Fr(this.accessCounter),
    //  endLifetime: Fr.ZERO,
    //};
    //this.newNoteHashes.push(traced);
    this.newNoteHashes.push(noteHash);
    this.incrementAccessCounter();
  }

  public traceNullifierCheck(storageAddress: Fr, nullifier: Fr, exists: boolean, isPending: boolean, leafIndex: Fr) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    const traced: TracedNullifierCheck = {
      callPointer: Fr.ZERO, // FIXME
      storageAddress,
      nullifier,
      exists,
      counter: new Fr(this.accessCounter),
      endLifetime: Fr.ZERO,
      isPending,
      leafIndex,
    };
    this.nullifierChecks.push(traced);
    this.incrementAccessCounter();
  }

  public traceNewNullifier(_storageAddress: Fr, nullifier: Fr) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    //const traced: TracedNullifier = {
    //  callPointer: Fr.ZERO,
    //  storageAddress,
    //  nullifier,
    //  counter: new Fr(this.accessCounter),
    //  endLifetime: Fr.ZERO,
    //};
    //this.newNullifiers.push(traced);
    this.newNullifiers.push(nullifier);
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
    mergeContractJournalMaps(this.publicStorageReads, incomingTrace.publicStorageReads);
    mergeContractJournalMaps(this.publicStorageWrites, incomingTrace.publicStorageWrites);
    // Merge new note hashes and nullifiers
    this.newNoteHashes = this.newNoteHashes.concat(incomingTrace.newNoteHashes);
    this.nullifierChecks = this.nullifierChecks.concat(incomingTrace.nullifierChecks);
    this.newNullifiers = this.newNullifiers.concat(incomingTrace.newNullifiers);
    // it is assumed that the incoming trace was initialized with this as parent, so accept counter
    this.accessCounter = incomingTrace.accessCounter;
  }

  /**
   * We want to keep track of all performed reads in the journal
   * This information is hinted to the avm circuit

   * @param contractAddress -
   * @param key -
   * @param value -
   */
  journalUpdate(map: Map<bigint, Map<bigint, Fr[]>>, contractAddress: Fr, key: Fr, value: Fr): void {
    let contractMap = map.get(contractAddress.toBigInt());
    if (!contractMap) {
      contractMap = new Map<bigint, Array<Fr>>();
      map.set(contractAddress.toBigInt(), contractMap);
    }

    let accessArray = contractMap.get(key.toBigInt());
    if (!accessArray) {
      accessArray = new Array<Fr>();
      contractMap.set(key.toBigInt(), accessArray);
    }
    accessArray.push(value);
  }

  // Create an instance of journalUpdate that appends to the read array
  private journalRead = this.journalUpdate.bind(this, this.publicStorageReads);
  // Create an instance of journalUpdate that appends to the writes array
  private journalWrite = this.journalUpdate.bind(this, this.publicStorageWrites);
}

/**
 * Merges two contract journalling maps together
 * For read maps, we just append the childMap arrays into the host map arrays, as the order is important
 *
 * @param hostMap - The map to be merged into
 * @param childMap - The map to be merged from
 */
function mergeContractJournalMaps(hostMap: Map<bigint, Map<bigint, Fr[]>>, childMap: Map<bigint, Map<bigint, Fr[]>>) {
  for (const [key, value] of childMap) {
    const map1Value = hostMap.get(key);
    if (!map1Value) {
      hostMap.set(key, value);
    } else {
      mergeStorageJournalMaps(map1Value, value);
    }
  }
}

/**
 * Merge two storage journalling maps together (for a particular contract).
 *
 * @param hostMap - The map to be merge into
 * @param childMap - The map to be merged from
 */
function mergeStorageJournalMaps(hostMap: Map<bigint, Fr[]>, childMap: Map<bigint, Fr[]>) {
  for (const [key, value] of childMap) {
    const readArr = hostMap.get(key);
    if (!readArr) {
      hostMap.set(key, value);
    } else {
      hostMap.set(key, readArr?.concat(...value));
    }
  }
}
