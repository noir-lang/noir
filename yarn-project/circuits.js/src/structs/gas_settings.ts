import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { GAS_SETTINGS_LENGTH } from '../constants.gen.js';
import { type UInt32 } from './shared.js';

/** Gas usage and fees limits set by the transaction sender for different dimensions and phases. */
export class GasSettings {
  constructor(
    public readonly da: DimensionGasSettings,
    public readonly l1: DimensionGasSettings,
    public readonly l2: DimensionGasSettings,
    public readonly inclusionFee: Fr,
  ) {}

  static empty() {
    return new GasSettings(
      DimensionGasSettings.empty(),
      DimensionGasSettings.empty(),
      DimensionGasSettings.empty(),
      Fr.ZERO,
    );
  }

  isEmpty() {
    return this.da.isEmpty() && this.l1.isEmpty() && this.l2.isEmpty() && this.inclusionFee.isZero();
  }

  equals(other: GasSettings) {
    return (
      this.da.equals(other.da) &&
      this.l1.equals(other.l1) &&
      this.l2.equals(other.l2) &&
      this.inclusionFee.equals(other.inclusionFee)
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader): GasSettings {
    const reader = BufferReader.asReader(buffer);
    return new GasSettings(
      reader.readObject(DimensionGasSettings),
      reader.readObject(DimensionGasSettings),
      reader.readObject(DimensionGasSettings),
      reader.readObject(Fr),
    );
  }

  toBuffer() {
    return serializeToBuffer(...GasSettings.getFields(this));
  }

  static fromFields(fields: Fr[] | FieldReader): GasSettings {
    const reader = FieldReader.asReader(fields);
    return new GasSettings(
      reader.readObject(DimensionGasSettings),
      reader.readObject(DimensionGasSettings),
      reader.readObject(DimensionGasSettings),
      reader.readField(),
    );
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...GasSettings.getFields(this));
    if (fields.length !== GAS_SETTINGS_LENGTH) {
      throw new Error(
        `Invalid number of fields for GasSettings. Expected ${GAS_SETTINGS_LENGTH} but got ${fields.length}`,
      );
    }
    return fields;
  }

  static getFields(fields: FieldsOf<GasSettings>) {
    return [fields.da, fields.l1, fields.l2, fields.inclusionFee] as const;
  }
}

/** Gas usage and fees limits set by the transaction sender for different phases on a specific dimension. */
export class DimensionGasSettings {
  constructor(
    public readonly gasLimit: UInt32,
    public readonly teardownGasLimit: UInt32,
    public readonly maxFeePerGas: Fr,
  ) {}

  static empty() {
    return new DimensionGasSettings(0, 0, Fr.ZERO);
  }

  isEmpty() {
    return this.gasLimit === 0 && this.maxFeePerGas.isZero() && this.teardownGasLimit === 0;
  }

  equals(other: DimensionGasSettings) {
    return (
      this.gasLimit === other.gasLimit &&
      this.maxFeePerGas.equals(other.maxFeePerGas) &&
      this.teardownGasLimit === other.teardownGasLimit
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader): DimensionGasSettings {
    const reader = BufferReader.asReader(buffer);
    return new DimensionGasSettings(reader.readNumber(), reader.readNumber(), reader.readObject(Fr));
  }

  toBuffer() {
    return serializeToBuffer(...DimensionGasSettings.getFields(this));
  }

  static fromFields(fields: Fr[] | FieldReader): DimensionGasSettings {
    const reader = FieldReader.asReader(fields);
    return new DimensionGasSettings(reader.readU32(), reader.readU32(), reader.readField());
  }

  toFields(): Fr[] {
    return serializeToFields(...DimensionGasSettings.getFields(this));
  }

  static getFields(fields: FieldsOf<DimensionGasSettings>) {
    return [fields.gasLimit, fields.teardownGasLimit, fields.maxFeePerGas] as const;
  }
}
