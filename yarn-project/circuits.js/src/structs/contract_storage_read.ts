import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { CONTRACT_STORAGE_READ_LENGTH } from '../constants.gen.js';

/**
 * Contract storage read operation on a specific contract.
 *
 * Note: Similar to `PublicDataRead` but it's from the POV of contract storage so we are not working with public data
 * tree leaf index but storage slot index.
 */
export class ContractStorageRead {
  constructor(
    /**
     * Storage slot we are reading from.
     */
    public readonly storageSlot: Fr,
    /**
     * Value read from the storage slot.
     */
    public readonly currentValue: Fr,
    /**
     * Side effect counter tracking position of this event in tx execution.
     */
    public readonly counter: number,
    /**
     * Contract address whose storage is being read.
     */
    public contractAddress?: AztecAddress, // TODO: Should not be optional. This is a temporary hack to silo the storage slot with the correct address for nested executions.
  ) {}

  static from(args: { storageSlot: Fr; currentValue: Fr; counter: number; contractAddress?: AztecAddress }) {
    return new ContractStorageRead(args.storageSlot, args.currentValue, args.counter, args.contractAddress);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.currentValue, new Fr(this.counter));
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractStorageRead(Fr.fromBuffer(reader), Fr.fromBuffer(reader), Fr.fromBuffer(reader).toNumber());
  }

  static empty() {
    return new ContractStorageRead(Fr.ZERO, Fr.ZERO, 0);
  }

  isEmpty() {
    return this.storageSlot.isZero() && this.currentValue.isZero() && this.counter == 0;
  }

  toFriendlyJSON() {
    return `Slot=${this.storageSlot.toFriendlyJSON()}: ${this.currentValue.toFriendlyJSON()}`;
  }

  toFields(): Fr[] {
    const fields = [this.storageSlot, this.currentValue, new Fr(this.counter)];
    if (fields.length !== CONTRACT_STORAGE_READ_LENGTH) {
      throw new Error(
        `Invalid number of fields for ContractStorageRead. Expected ${CONTRACT_STORAGE_READ_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): ContractStorageRead {
    const reader = FieldReader.asReader(fields);

    const storageSlot = reader.readField();
    const currentValue = reader.readField();
    const counter = reader.readField().toNumber();

    return new ContractStorageRead(storageSlot, currentValue, counter);
  }
}
