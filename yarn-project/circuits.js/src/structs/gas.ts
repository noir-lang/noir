import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

import { type UInt32 } from './shared.js';

export const GasDimensions = ['da', 'l1', 'l2'] as const;
export type GasDimensions = (typeof GasDimensions)[number];

/** Gas amounts in each dimension. */
export class Gas {
  constructor(public readonly daGas: UInt32, public readonly l1Gas: UInt32, public readonly l2Gas: UInt32) {}

  clone(): Gas {
    return new Gas(this.daGas, this.l1Gas, this.l2Gas);
  }

  get(dimension: GasDimensions) {
    return this[`${dimension}Gas`];
  }

  equals(other: Gas) {
    return this.daGas === other.daGas && this.l1Gas === other.l1Gas && this.l2Gas === other.l2Gas;
  }

  static from(fields: FieldsOf<Gas>) {
    return new Gas(fields.daGas, fields.l1Gas, fields.l2Gas);
  }

  static empty() {
    return new Gas(0, 0, 0);
  }

  /** Returns large enough gas amounts for testing purposes. */
  static test() {
    return new Gas(1e9, 1e9, 1e9);
  }

  isEmpty() {
    return this.daGas === 0 && this.l1Gas === 0 && this.l2Gas === 0;
  }

  static fromBuffer(buffer: Buffer | BufferReader): Gas {
    const reader = BufferReader.asReader(buffer);
    return new Gas(reader.readNumber(), reader.readNumber(), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.daGas, this.l1Gas, this.l2Gas);
  }

  [inspect.custom]() {
    return `Gas { daGas=${this.daGas} l1Gas=${this.l1Gas} l2Gas=${this.l2Gas} }`;
  }

  add(other: Gas) {
    return new Gas(this.daGas + other.daGas, this.l1Gas + other.l1Gas, this.l2Gas + other.l2Gas);
  }

  sub(other: Gas) {
    return new Gas(this.daGas - other.daGas, this.l1Gas - other.l1Gas, this.l2Gas - other.l2Gas);
  }

  mul(scalar: number) {
    return new Gas(Math.ceil(this.daGas * scalar), Math.ceil(this.l1Gas * scalar), Math.ceil(this.l2Gas * scalar));
  }

  toFields() {
    return serializeToFields(this.daGas, this.l1Gas, this.l2Gas);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new Gas(reader.readU32(), reader.readU32(), reader.readU32());
  }
}
