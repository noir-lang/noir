import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { TX_CONTEXT_LENGTH } from '../constants.gen.js';
import { GasSettings } from './gas_settings.js';

/**
 * Transaction context.
 */
export class TxContext {
  public chainId: Fr;
  public version: Fr;

  constructor(
    /** Chain ID of the transaction. Here for replay protection. */
    chainId: Fr | number | bigint,
    /** Version of the transaction. Here for replay protection. */
    version: Fr | number | bigint,
    /** Gas limits for this transaction. */
    public gasSettings: GasSettings,
  ) {
    this.chainId = new Fr(chainId);
    this.version = new Fr(version);
  }

  getSize() {
    return this.chainId.size + this.version.size + this.gasSettings.getSize();
  }

  clone() {
    return new TxContext(this.chainId, this.version, this.gasSettings.clone());
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...TxContext.getFields(this));
  }

  static fromFields(fields: Fr[] | FieldReader): TxContext {
    const reader = FieldReader.asReader(fields);
    return new TxContext(reader.readField(), reader.readField(), reader.readObject(GasSettings));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...TxContext.getFields(this));
    if (fields.length !== TX_CONTEXT_LENGTH) {
      throw new Error(`Invalid number of fields for TxContext. Expected ${TX_CONTEXT_LENGTH}, got ${fields.length}`);
    }
    return fields;
  }

  /**
   * Deserializes TxContext from a buffer or reader.
   * @param buffer - Buffer to read from.
   * @returns The TxContext.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxContext {
    const reader = BufferReader.asReader(buffer);
    return new TxContext(Fr.fromBuffer(reader), Fr.fromBuffer(reader), reader.readObject(GasSettings));
  }

  static empty(chainId: Fr | number = 0, version: Fr | number = 0) {
    return new TxContext(new Fr(chainId), new Fr(version), GasSettings.empty());
  }

  isEmpty(): boolean {
    return this.chainId.isZero() && this.version.isZero() && this.gasSettings.isEmpty();
  }

  /**
   * Create a new instance from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A new instance.
   */
  static from(fields: FieldsOf<TxContext>): TxContext {
    return new TxContext(...TxContext.getFields(fields));
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<TxContext>) {
    return [fields.chainId, fields.version, fields.gasSettings] as const;
  }
}
