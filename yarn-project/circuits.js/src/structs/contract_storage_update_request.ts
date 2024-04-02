import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH } from '../constants.gen.js';

/**
 * Contract storage update request for a slot on a specific contract.
 *
 * Note: Similar to `PublicDataUpdateRequest` but it's from the POV of contract storage so we are not working with
 * public data tree leaf index but storage slot index.
 */
export class ContractStorageUpdateRequest {
  constructor(
    /**
     * Storage slot we are updating.
     */
    public readonly storageSlot: Fr,
    /**
     * New value of the storage slot.
     */
    public readonly newValue: Fr,
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractStorageUpdateRequest(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  /**
   * Create PublicCallRequest from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PublicCallRequest object.
   */
  static from(fields: FieldsOf<ContractStorageUpdateRequest>): ContractStorageUpdateRequest {
    return new ContractStorageUpdateRequest(...ContractStorageUpdateRequest.getFields(fields));
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<ContractStorageUpdateRequest>) {
    return [fields.storageSlot, fields.newValue, fields.sideEffectCounter] as const;
  }

  static empty() {
    return new ContractStorageUpdateRequest(Fr.ZERO, Fr.ZERO);
  }

  isEmpty() {
    return this.storageSlot.isZero() && this.newValue.isZero();
  }

  toFriendlyJSON() {
    return `Slot=${this.storageSlot.toFriendlyJSON()}: ${this.newValue.toFriendlyJSON()}`;
  }

  toFields(): Fr[] {
    const fields = [this.storageSlot, this.newValue];
    if (fields.length !== CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH) {
      throw new Error(
        `Invalid number of fields for ContractStorageUpdateRequest. Expected ${CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): ContractStorageUpdateRequest {
    const reader = FieldReader.asReader(fields);

    const storageSlot = reader.readField();
    const newValue = reader.readField();

    return new ContractStorageUpdateRequest(storageSlot, newValue);
  }
}
