import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import {
  DEFAULT_GAS_LIMIT,
  DEFAULT_INCLUSION_FEE,
  DEFAULT_MAX_FEE_PER_GAS,
  DEFAULT_TEARDOWN_GAS_LIMIT,
  GAS_SETTINGS_LENGTH,
} from '../constants.gen.js';
import { Gas, GasDimensions } from './gas.js';
import { GasFees } from './gas_fees.js';

/** Gas usage and fees limits set by the transaction sender for different dimensions and phases. */
export class GasSettings {
  constructor(
    public readonly gasLimits: Gas,
    public readonly teardownGasLimits: Gas,
    public readonly maxFeesPerGas: GasFees,
    public readonly inclusionFee: Fr,
  ) {}

  static from(args: {
    gasLimits: FieldsOf<Gas>;
    teardownGasLimits: FieldsOf<Gas>;
    maxFeesPerGas: FieldsOf<GasFees>;
    inclusionFee: Fr;
  }) {
    return new GasSettings(
      Gas.from(args.gasLimits),
      Gas.from(args.teardownGasLimits),
      GasFees.from(args.maxFeesPerGas),
      args.inclusionFee,
    );
  }

  clone() {
    return new GasSettings(
      this.gasLimits.clone(),
      this.teardownGasLimits.clone(),
      this.maxFeesPerGas.clone(),
      this.inclusionFee,
    );
  }

  /** Returns the maximum fee to be paid according to gas limits and max fees set. */
  getFeeLimit() {
    return GasDimensions.reduce(
      (acc, dimension) =>
        this.maxFeesPerGas
          .get(dimension)
          .mul(new Fr(this.gasLimits.get(dimension)))
          .add(acc),
      Fr.ZERO,
    ).add(this.inclusionFee);
  }

  /** Zero-value gas settings. */
  static empty() {
    return new GasSettings(Gas.empty(), Gas.empty(), GasFees.empty(), Fr.ZERO);
  }

  /** Default gas settings to use when user has not provided them. */
  static default() {
    return new GasSettings(
      new Gas(DEFAULT_GAS_LIMIT, DEFAULT_GAS_LIMIT),
      new Gas(DEFAULT_TEARDOWN_GAS_LIMIT, DEFAULT_TEARDOWN_GAS_LIMIT),
      new GasFees(new Fr(DEFAULT_MAX_FEE_PER_GAS), new Fr(DEFAULT_MAX_FEE_PER_GAS)),
      new Fr(DEFAULT_INCLUSION_FEE),
    );
  }

  /** Gas settings to use for simulating a contract call. */
  static simulation() {
    return GasSettings.default();
  }

  isEmpty() {
    return (
      this.gasLimits.isEmpty() &&
      this.teardownGasLimits.isEmpty() &&
      this.maxFeesPerGas.isEmpty() &&
      this.inclusionFee.isZero()
    );
  }

  equals(other: GasSettings) {
    return (
      this.gasLimits.equals(other.gasLimits) &&
      this.teardownGasLimits.equals(other.teardownGasLimits) &&
      this.maxFeesPerGas.equals(other.maxFeesPerGas) &&
      this.inclusionFee.equals(other.inclusionFee)
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader): GasSettings {
    const reader = BufferReader.asReader(buffer);
    return new GasSettings(
      reader.readObject(Gas),
      reader.readObject(Gas),
      reader.readObject(GasFees),
      reader.readObject(Fr),
    );
  }

  toBuffer() {
    return serializeToBuffer(...GasSettings.getFields(this));
  }

  static fromFields(fields: Fr[] | FieldReader): GasSettings {
    const reader = FieldReader.asReader(fields);
    return new GasSettings(
      reader.readObject(Gas),
      reader.readObject(Gas),
      reader.readObject(GasFees),
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
    return [fields.gasLimits, fields.teardownGasLimits, fields.maxFeesPerGas, fields.inclusionFee] as const;
  }

  /** Returns total gas limits. */
  getLimits(): Gas {
    return this.gasLimits;
  }

  /** Returns how much gas is available for execution of setup and app phases (ie total limit minus teardown). */
  getInitialAvailable(): Gas {
    return this.gasLimits.sub(this.teardownGasLimits);
  }

  /** Returns how much gas is available for execution of teardown phase. */
  getTeardownLimits(): Gas {
    return this.teardownGasLimits;
  }
}
