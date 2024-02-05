import { Fr } from '@aztec/foundation/fields';

import { HostStorage } from './host_storage.js';

/**
 * Data held within the journal
 */
export type JournalData = {
  newNoteHashes: Fr[];
  newNullifiers: Fr[];
  newL1Messages: Fr[][];
  newLogs: Fr[][];

  /** contract address -\> key -\> value */
  currentStorageValue: Map<bigint, Map<bigint, Fr>>;

  /** contract address -\> key -\> value[] (stored in order of access) */
  storageWrites: Map<bigint, Map<bigint, Fr[]>>;
  /** contract address -\> key -\> value[] (stored in order of access) */
  storageReads: Map<bigint, Map<bigint, Fr[]>>;
};

/**
 * A cache of the current state of the AVM
 * The interpreter should make any state queries through this object
 *
 * When a nested context succeeds, it's journal is merge into the parent
 * When a call fails, it's journal is discarded and the parent is used from this point forward
 * When a call succeeds's we can merge a child into its parent
 */
export class AvmWorldStateJournal {
  /** Reference to node storage */
  public readonly hostStorage: HostStorage;

  // Reading state - must be tracked for vm execution
  // contract address -> key -> value[] (array stored in order of reads)
  private storageReads: Map<bigint, Map<bigint, Fr[]>> = new Map();
  private storageWrites: Map<bigint, Map<bigint, Fr[]>> = new Map();

  // New written state
  private newNoteHashes: Fr[] = [];
  private newNullifiers: Fr[] = [];

  // New Substate
  private newL1Messages: Fr[][] = [];
  private newLogs: Fr[][] = [];

  // contract address -> key -> value
  private currentStorageValue: Map<bigint, Map<bigint, Fr>> = new Map();

  private parentJournal: AvmWorldStateJournal | undefined;

  constructor(hostStorage: HostStorage, parentJournal?: AvmWorldStateJournal) {
    this.hostStorage = hostStorage;
    this.parentJournal = parentJournal;
  }

  /**
   * Create a new world state journal forked from this one
   */
  public fork() {
    return new AvmWorldStateJournal(this.hostStorage, this);
  }

  /**
   * Write storage into journal
   *
   * @param contractAddress -
   * @param key -
   * @param value -
   */
  public writeStorage(contractAddress: Fr, key: Fr, value: Fr) {
    let contractMap = this.currentStorageValue.get(contractAddress.toBigInt());
    if (!contractMap) {
      contractMap = new Map();
      this.currentStorageValue.set(contractAddress.toBigInt(), contractMap);
    }
    contractMap.set(key.toBigInt(), value);

    // We want to keep track of all performed writes in the journal
    this.journalWrite(contractAddress, key, value);
  }

  /**
   * Read storage from journal
   * Read from host storage on cache miss
   *
   * @param contractAddress -
   * @param key -
   * @returns current value
   */
  public async readStorage(contractAddress: Fr, key: Fr): Promise<Fr> {
    // - We first try this journal's storage cache ( if written to before in this call frame )
    // - Then we try the parent journal's storage cache ( if it exists ) ( written to earlier in this block )
    // - Finally we try the host storage ( a trip to the database )

    // Do not early return as we want to keep track of reads in this.storageReads
    let value = this.currentStorageValue.get(contractAddress.toBigInt())?.get(key.toBigInt());
    if (!value && this.parentJournal) {
      value = await this.parentJournal?.readStorage(contractAddress, key);
    }
    if (!value) {
      value = await this.hostStorage.publicStateDb.storageRead(contractAddress, key);
    }

    this.journalRead(contractAddress, key, value);
    return Promise.resolve(value);
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
  private journalRead = this.journalUpdate.bind(this, this.storageReads);
  // Create an instance of journalUpdate that appends to the writes array
  private journalWrite = this.journalUpdate.bind(this, this.storageWrites);

  public writeNoteHash(noteHash: Fr) {
    this.newNoteHashes.push(noteHash);
  }

  public writeL1Message(message: Fr[]) {
    this.newL1Messages.push(message);
  }

  public writeNullifier(nullifier: Fr) {
    this.newNullifiers.push(nullifier);
  }

  public writeLog(log: Fr[]) {
    this.newLogs.push(log);
  }

  /**
   * Accept nested world state, merging in its journal, and accepting its state modifications
   * - Utxo objects are concatenated
   * - Public state changes are merged, with the value in the incoming journal taking precedent
   * - Public state journals (r/w logs), with the accessing being appended in chronological order
   */
  public acceptNestedWorldState(nestedJournal: AvmWorldStateJournal) {
    // Merge UTXOs
    this.newNoteHashes = this.newNoteHashes.concat(nestedJournal.newNoteHashes);
    this.newL1Messages = this.newL1Messages.concat(nestedJournal.newL1Messages);
    this.newNullifiers = this.newNullifiers.concat(nestedJournal.newNullifiers);
    this.newLogs = this.newLogs.concat(nestedJournal.newLogs);

    // Merge Public State
    mergeCurrentValueMaps(this.currentStorageValue, nestedJournal.currentStorageValue);

    // Merge storage read and write journals
    mergeContractJournalMaps(this.storageReads, nestedJournal.storageReads);
    mergeContractJournalMaps(this.storageWrites, nestedJournal.storageWrites);
  }

  /**
   * Reject nested world state, merging in its journal, but not accepting its state modifications
   * - Utxo objects are concatenated
   * - Public state changes are dropped
   * - Public state journals (r/w logs) are maintained, with the accessing being appended in chronological order
   */
  public rejectNestedWorldState(nestedJournal: AvmWorldStateJournal) {
    // Merge storage read and write journals
    mergeContractJournalMaps(this.storageReads, nestedJournal.storageReads);
    mergeContractJournalMaps(this.storageWrites, nestedJournal.storageWrites);
  }

  /**
   * Access the current state of the journal
   *
   * @returns a JournalData object
   */
  public flush(): JournalData {
    return {
      newNoteHashes: this.newNoteHashes,
      newNullifiers: this.newNullifiers,
      newL1Messages: this.newL1Messages,
      newLogs: this.newLogs,
      currentStorageValue: this.currentStorageValue,
      storageReads: this.storageReads,
      storageWrites: this.storageWrites,
    };
  }
}

/**
 * Merges two contract current value together
 * Where childMap keys will take precedent over the hostMap
 * The assumption being that the child map is created at a later time
 * And thus contains more up to date information
 *
 * @param hostMap - The map to be merged into
 * @param childMap - The map to be merged from
 */
function mergeCurrentValueMaps(hostMap: Map<bigint, Map<bigint, Fr>>, childMap: Map<bigint, Map<bigint, Fr>>) {
  for (const [key, value] of childMap) {
    const map1Value = hostMap.get(key);
    if (!map1Value) {
      hostMap.set(key, value);
    } else {
      mergeStorageCurrentValueMaps(map1Value, value);
    }
  }
}

/**
 * @param hostMap - The map to be merge into
 * @param childMap - The map to be merged from
 */
function mergeStorageCurrentValueMaps(hostMap: Map<bigint, Fr>, childMap: Map<bigint, Fr>) {
  for (const [key, value] of childMap) {
    hostMap.set(key, value);
  }
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
