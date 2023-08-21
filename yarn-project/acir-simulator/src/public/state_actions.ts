import { ContractStorageRead, ContractStorageUpdateRequest } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { PublicStateDB } from './db.js';

/**
 * Implements read/write operations on a contract public storage, collecting
 * all read and update operations, and collapsing them into a single
 * read or update per slot.
 */
export class ContractStorageActionsCollector {
  // Map from slot to first read value
  private readonly contractStorageReads: Map<
    bigint,
    { /** The value read. */ currentValue: Fr; /** Side effect counter. */ sideEffectCounter: number }
  > = new Map();

  // Map from slot to first read value and latest updated value
  private readonly contractStorageUpdateRequests: Map<
    bigint,
    {
      /** The old value. */ oldValue: Fr;
      /** The updated value. */ newValue: Fr;
      /** Side effect counter. */ sideEffectCounter: number;
    }
  > = new Map();

  constructor(private db: PublicStateDB, private address: AztecAddress) {}

  /**
   * Returns the current value of a slot according to the latest update request for it,
   * falling back to the public db. Collects the operation in storage reads,
   * as long as there is no existing update request.
   * @param storageSlot - Slot to check.
   * @param sideEffectCounter - Side effect counter associated with this storage action.
   * @returns The current value as affected by all update requests so far.
   */
  public async read(storageSlot: Fr, sideEffectCounter: number): Promise<Fr> {
    const slot = storageSlot.value;
    const updateRequest = this.contractStorageUpdateRequests.get(slot);
    if (updateRequest) return updateRequest.newValue;
    const read = this.contractStorageReads.get(slot);
    if (read) return read.currentValue;
    const value = await this.db.storageRead(this.address, storageSlot);
    this.contractStorageReads.set(slot, { currentValue: value, sideEffectCounter });
    return value;
  }

  /**
   * Sets a new value for a slot in the internal update requests cache,
   * clearing any previous storage read or update operation for the same slot.
   * @param storageSlot - Slot to write to.
   * @param newValue - Value to write to it.
   * @param sideEffectCounter - Side effect counter associated with this storage action.
   */
  public async write(storageSlot: Fr, newValue: Fr, sideEffectCounter: number): Promise<void> {
    const slot = storageSlot.value;
    const updateRequest = this.contractStorageUpdateRequests.get(slot);
    if (updateRequest) {
      this.contractStorageUpdateRequests.set(slot, { oldValue: updateRequest.oldValue, newValue, sideEffectCounter });
      return;
    }

    const read = this.contractStorageReads.get(slot);
    if (read) {
      this.contractStorageReads.delete(slot);
      this.contractStorageUpdateRequests.set(slot, { oldValue: read.currentValue, newValue, sideEffectCounter });
      return;
    }

    const oldValue = await this.db.storageRead(this.address, storageSlot);
    this.contractStorageUpdateRequests.set(slot, { oldValue, newValue, sideEffectCounter });
    return;
  }

  /**
   * Returns all storage reads and update requests performed.
   * @returns All storage read and update requests.
   */
  public collect(): [ContractStorageRead[], ContractStorageUpdateRequest[]] {
    const reads = Array.from(this.contractStorageReads.entries()).map(([slot, valueAndCounter]) =>
      ContractStorageRead.from({
        storageSlot: new Fr(slot),
        ...valueAndCounter,
      }),
    );

    const updateRequests = Array.from(this.contractStorageUpdateRequests.entries()).map(([slot, valuesAndCounter]) =>
      ContractStorageUpdateRequest.from({
        storageSlot: new Fr(slot),
        ...valuesAndCounter,
      }),
    );

    return [reads, updateRequests];
  }
}
