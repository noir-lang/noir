import { Fr } from '@aztec/foundation/fields';

import { HostStorage } from './host_storage.js';

// TODO: all of the data that comes out of the avm ready for write should be in this format
/** - */
export type JournalData = {
  /** - */
  newCommitments: Fr[];
  /** - */
  newL1Message: Fr[];
  /** - */
  storageWrites: { [key: string]: { [key: string]: Fr } };
};

// This persists for an entire block
// Each transaction should have its own journal that gets appended to this one upon success
/** - */
export class AvmJournal {
  // TODO: should we make private?
  /** - */
  public readonly hostStorage: HostStorage;

  // We need to keep track of the following
  // - State reads
  // - State updates
  // - New Commitments
  // - Commitment reads

  private newCommitments: Fr[] = [];
  private newL1Message: Fr[] = [];

  // TODO: type this structure -> contract address -> key -> value
  private storageWrites: { [key: string]: { [key: string]: Fr } } = {};

  constructor(hostStorage: HostStorage) {
    this.hostStorage = hostStorage;
  }

  // TODO: work on the typing
  /**
   * -
   * @param contractAddress -
   * @param key -
   * @param value -
   */
  public writeStorage(contractAddress: Fr, key: Fr, value: Fr) {
    // TODO: do we want this map to be ordered -> is there performance upside to this?
    this.storageWrites[contractAddress.toString()][key.toString()] = value;
  }

  /**
   * -
   * @param contractAddress -
   * @param key -
   */
  public readStorage(contractAddress: Fr, key: Fr) {
    const cachedValue = this.storageWrites[contractAddress.toString()][key.toString()];
    if (cachedValue) {
      return cachedValue;
    }
    return this.hostStorage.stateDb.storageRead(contractAddress, key);
  }

  /**
   * -
   * @param commitment -
   */
  public writeCommitment(commitment: Fr) {
    this.newCommitments.push(commitment);
  }

  /**
   * -
   * @param message -
   */
  public writeL1Message(message: Fr) {
    this.newL1Message.push(message);
  }

  // TODO: This function will merge two journals together -> the new head of the chain
  /**
   * -
   * @param journal -
   */
  public mergeJournal(journal: AvmJournal) {
    // TODO: This function will
    void journal;
  }

  /**
   * -
   */
  public flush(): JournalData {
    return {
      newCommitments: this.newCommitments,
      newL1Message: this.newL1Message,
      storageWrites: this.storageWrites,
    };
  }
}
