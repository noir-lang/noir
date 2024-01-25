import { Fr } from '@aztec/foundation/fields';

import { RootJournalCannotBeMerged } from './errors.js';
import { HostStorage } from './host_storage.js';

/**
 * Data held within the journal
 */
export type JournalData = {
  /** - */
  newCommitments: Fr[];
  /** - */
  newL1Messages: Fr[];
  /** - */
  newNullifiers: Fr[];
  /** contract address -\> key -\> value */
  storageWrites: Map<bigint, Map<bigint, Fr>>;
};

/**
 * A cache of the current state of the AVM
 * The interpreter should make any state queries through this object
 *
 * When a sub context's call succeeds, it's journal is merge into the parent
 * When a a call fails, it's journal is discarded and the parent is used from this point forward
 * When a call succeeds's we can merge a child into its parent
 */
export class AvmJournal {
  /** Reference to node storage */
  public readonly hostStorage: HostStorage;

  // Reading state - must be tracked for vm execution
  // contract address -> key -> value
  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3999)
  private storageReads: Map<bigint, Map<bigint, Fr>> = new Map();

  // New written state
  private newCommitments: Fr[] = [];
  private newNullifiers: Fr[] = [];
  private newL1Message: Fr[] = [];

  // New Substrate
  private newLogs: Fr[][] = [];

  // contract address -> key -> value
  private storageWrites: Map<bigint, Map<bigint, Fr>> = new Map();

  private parentJournal: AvmJournal | undefined;

  constructor(hostStorage: HostStorage, parentJournal?: AvmJournal) {
    this.hostStorage = hostStorage;
    this.parentJournal = parentJournal;
  }

  /**
   * Create a new root journal, without a parent
   * @param hostStorage -
   */
  public static rootJournal(hostStorage: HostStorage) {
    return new AvmJournal(hostStorage);
  }

  /**
   * Create a new journal from a parent
   * @param parentJournal -
   */
  public static branchParent(parentJournal: AvmJournal) {
    return new AvmJournal(parentJournal.hostStorage, parentJournal);
  }

  /**
   * Write storage into journal
   *
   * @param contractAddress -
   * @param key -
   * @param value -
   */
  public writeStorage(contractAddress: Fr, key: Fr, value: Fr) {
    let contractMap = this.storageWrites.get(contractAddress.toBigInt());
    if (!contractMap) {
      contractMap = new Map();
      this.storageWrites.set(contractAddress.toBigInt(), contractMap);
    }
    contractMap.set(key.toBigInt(), value);
  }

  /**
   * Read storage from journal
   * Read from host storage on cache miss
   *
   * @param contractAddress -
   * @param key -
   * @returns current value
   */
  public readStorage(contractAddress: Fr, key: Fr): Promise<Fr> {
    const cachedValue = this.storageWrites.get(contractAddress.toBigInt())?.get(key.toBigInt());
    if (cachedValue) {
      return Promise.resolve(cachedValue);
    }
    if (this.parentJournal) {
      return this.parentJournal?.readStorage(contractAddress, key);
    }
    return this.hostStorage.publicStateDb.storageRead(contractAddress, key);
  }

  /** -
   * @param commitment -
   */
  public writeCommitment(commitment: Fr) {
    this.newCommitments.push(commitment);
  }

  /** -
   * @param message -
   */
  public writeL1Message(message: Fr) {
    this.newL1Message.push(message);
  }

  /** -
   * @param nullifier -
   */
  public writeNullifier(nullifier: Fr) {
    this.newNullifiers.push(nullifier);
  }

  /**
   * Merge Journal into parent
   * - Utxo objects are concatenated
   * - Public state is merged, with the value in the incoming journal taking precedent
   */
  public mergeWithParent() {
    if (!this.parentJournal) {
      throw new RootJournalCannotBeMerged();
    }

    const incomingFlush = this.flush();

    // Merge UTXOs
    this.parentJournal.newCommitments = this.parentJournal.newCommitments.concat(incomingFlush.newCommitments);
    this.parentJournal.newL1Message = this.parentJournal.newL1Message.concat(incomingFlush.newL1Messages);
    this.parentJournal.newNullifiers = this.parentJournal.newNullifiers.concat(incomingFlush.newNullifiers);

    // Merge Public State
    mergeContractMaps(this.parentJournal.storageWrites, incomingFlush.storageWrites);
  }

  /** Access the current state of the journal
   *
   * @returns a JournalData object that can be used to write to the storage
   */
  public flush(): JournalData {
    return {
      newCommitments: this.newCommitments,
      newL1Messages: this.newL1Message,
      newNullifiers: this.newNullifiers,
      storageWrites: this.storageWrites,
    };
  }
}

/**
 * Merges two contract maps together
 * Where childMap keys will take precedent over the hostMap
 * The assumption being that the child map is created at a later time
 * And thus contains more up to date information
 *
 * @param hostMap - The map to be merged into
 * @param childMap - The map to be merged from
 */
function mergeContractMaps(hostMap: Map<bigint, Map<bigint, Fr>>, childMap: Map<bigint, Map<bigint, Fr>>) {
  for (const [key, value] of childMap) {
    const map1Value = hostMap.get(key);
    if (!map1Value) {
      hostMap.set(key, value);
    } else {
      mergeStorageMaps(map1Value, value);
    }
  }
}

/**
 *
 * @param hostMap - The map to be merge into
 * @param childMap - The map to be merged from
 */
function mergeStorageMaps(hostMap: Map<bigint, Fr>, childMap: Map<bigint, Fr>) {
  for (const [key, value] of childMap) {
    hostMap.set(key, value);
  }
}
