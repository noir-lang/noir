import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { GAS_SETTINGS_LENGTH } from '../constants.gen.js';
import { Gas } from './gas.js';
import { type UInt32 } from './shared.js';

/** Gas usage and fees limits set by the transaction sender for different dimensions and phases. */
export class GasSettings {
  constructor(
    public readonly da: DimensionGasSettings,
    public readonly l1: DimensionGasSettings,
    public readonly l2: DimensionGasSettings,
    public readonly inclusionFee: Fr,
  ) {}

  static new(args: {
    da: FieldsOf<DimensionGasSettings>;
    l1: FieldsOf<DimensionGasSettings>;
    l2: FieldsOf<DimensionGasSettings>;
    inclusionFee: Fr;
  }) {
    return new GasSettings(
      DimensionGasSettings.from(args.da),
      DimensionGasSettings.from(args.l1),
      DimensionGasSettings.from(args.l2),
      args.inclusionFee,
    );
  }

  /** Returns the maximum fee to be paid according to gas limits and max fees set. */
  getFeeLimit() {
    return [this.da, this.l1, this.l2]
      .reduce((acc, dimension) => acc.add(dimension.getFeeLimit()), Fr.ZERO)
      .add(this.inclusionFee);
  }

  /** Zero-value gas settings. */
  static empty() {
    return new GasSettings(
      DimensionGasSettings.empty(),
      DimensionGasSettings.empty(),
      DimensionGasSettings.empty(),
      Fr.ZERO,
    );
  }

  /** Default gas settings to use when user has not provided them. */
  static default() {
    return new GasSettings(
      DimensionGasSettings.default(),
      DimensionGasSettings.default(),
      DimensionGasSettings.default(),
      Fr.ONE,
    );
  }

  /** Gas settings to use for simulating a contract call. */
  static simulation() {
    return new GasSettings(
      DimensionGasSettings.simulation(),
      DimensionGasSettings.simulation(),
      DimensionGasSettings.simulation(),
      Fr.ONE,
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

  /** Returns total gas limits. */
  getLimits(): Gas {
    return new Gas(this.da.gasLimit, this.l1.gasLimit, this.l2.gasLimit);
  }

  /** Returns how much gas is available for execution of setup and app phases (ie total limit minus teardown). */
  getInitialAvailable(): Gas {
    return new Gas(
      this.da.gasLimit - this.da.teardownGasLimit,
      this.l1.gasLimit - this.l1.teardownGasLimit,
      this.l2.gasLimit - this.l2.teardownGasLimit,
    );
  }

  /** Returns how much gas is available for execution of teardown phase. */
  getTeardownLimits(): Gas {
    return new Gas(this.da.teardownGasLimit, this.l1.teardownGasLimit, this.l2.teardownGasLimit);
  }
}

/** Gas usage and fees limits set by the transaction sender for different phases on a specific dimension. */
export class DimensionGasSettings {
  constructor(
    public readonly gasLimit: UInt32,
    public readonly teardownGasLimit: UInt32,
    public readonly maxFeePerGas: Fr,
  ) {
    if (teardownGasLimit > gasLimit) {
      throw new Error(`Teardown gas limit ${teardownGasLimit} is greater than gas limit ${gasLimit}`);
    }
  }

  static default() {
    return new DimensionGasSettings(1e9, 1e8, Fr.ONE);
  }

  static simulation() {
    return new DimensionGasSettings(1e9, 1e8, Fr.ONE);
  }

  getFeeLimit() {
    return this.maxFeePerGas.mul(new Fr(this.gasLimit + this.teardownGasLimit));
  }

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

  static from(fields: FieldsOf<DimensionGasSettings>) {
    return new DimensionGasSettings(fields.gasLimit, fields.teardownGasLimit, fields.maxFeePerGas);
  }
}
