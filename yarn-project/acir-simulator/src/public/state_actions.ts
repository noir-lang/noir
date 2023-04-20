import { AztecAddress, Fr } from '@aztec/foundation';
import { PublicDB } from './db.js';
import { StateRead, StateTransition } from '@aztec/circuits.js';

/**
 * Implements state read/write operations on a contract public storage, collecting
 * all state read and transitions operations, and collapsing them into a single
 * read or transition per slot.
 */
export class StateActionsCollector {
  // Map from slot to first read value
  private readonly stateReads: Map<bigint, { value: Fr }> = new Map();

  // Map from slot to first read value and latest updated value
  private readonly stateTransitions: Map<bigint, { oldValue: Fr; newValue: Fr }> = new Map();

  constructor(private db: PublicDB, private address: AztecAddress) {}

  /**
   * Returns the current value of a slot according to the latest transition for it,
   * falling back to the public db. Collects the operation in state reads,
   * as long as there is no existing state transition.
   * @param storageSlot - slot to check
   * @returns The current value as affected by all state transitions so far.
   */
  public async read(storageSlot: Fr): Promise<Fr> {
    const slot = storageSlot.value;
    const transition = this.stateTransitions.get(slot);
    if (transition) return transition.newValue;
    const read = this.stateReads.get(slot);
    if (read) return read.value;
    const value = await this.db.storageRead(this.address, storageSlot);
    this.stateReads.set(slot, { value });
    return value;
  }

  /**
   * Sets a new value for a slot in the internal state transitions cache,
   * clearing any previous state read or transition operation for the same slot.
   * @param storageSlot - slot to write to
   * @param newValue - value to write to it
   */
  public async write(storageSlot: Fr, newValue: Fr): Promise<void> {
    const slot = storageSlot.value;
    const transition = this.stateTransitions.get(slot);
    if (transition) {
      this.stateTransitions.set(slot, { oldValue: transition.oldValue, newValue });
      return;
    }

    const read = this.stateReads.get(slot);
    if (read) {
      this.stateReads.delete(slot);
      this.stateTransitions.set(slot, { oldValue: read.value, newValue });
      return;
    }

    const oldValue = await this.db.storageRead(this.address, storageSlot);
    this.stateTransitions.set(slot, { oldValue, newValue });
    return;
  }

  /**
   * Returns all state read and transitions performed.
   * @returns all state read and transitions
   */
  public collect(): [StateRead[], StateTransition[]] {
    const reads = Array.from(this.stateReads.entries()).map(([slot, value]) =>
      StateRead.from({
        storageSlot: new Fr(slot),
        ...value,
      }),
    );

    const transitions = Array.from(this.stateTransitions.entries()).map(([slot, values]) =>
      StateTransition.from({
        storageSlot: new Fr(slot),
        ...values,
      }),
    );

    return [reads, transitions];
  }
}
