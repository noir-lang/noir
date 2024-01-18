import { BlockHeader } from '@aztec/circuits.js';

import { AvmJournal, HostStorage } from './journal/index.js';

/**
 * The Avm State Manager is the interpreter's interface to the node's state
 * It creates revertible views into the node state and manages the current call's journal
 */
export class AvmStateManager {
  /** - */
  public readonly blockHeader: BlockHeader;

  /**
   * Journal keeps track of pending state changes
   */
  public readonly journal: AvmJournal;

  constructor(blockHeader: BlockHeader, journal: AvmJournal) {
    this.blockHeader = blockHeader;
    this.journal = journal;
  }

  /**
   * Create a base state root manager
   * - this should be created by the highest level item where the state
   *   can be reverted
   * @param blockHeader -
   * @param hostStorage - An immutable view into the node db
   * @returns Avm State Manager
   */
  public static rootStateManager(blockHeader: BlockHeader, hostStorage: HostStorage): AvmStateManager {
    const journal = AvmJournal.rootJournal(hostStorage);
    return new AvmStateManager(blockHeader, journal);
  }

  /**
   *  Avm State
   * @param parent - Avm state manager with a forked journal
   * @returns
   */
  public static forkStateManager(parent: AvmStateManager): AvmStateManager {
    const journal = AvmJournal.branchParent(parent.journal);
    return new AvmStateManager(parent.blockHeader, journal);
  }
}
