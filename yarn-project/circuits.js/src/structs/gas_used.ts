import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

import { type UInt32 } from './shared.js';

/** Gas in each dimension used so far in the context of the current transaction or phase. */
export class GasUsed {
  constructor(public readonly daGas: UInt32, public readonly l1Gas: UInt32, public readonly l2Gas: UInt32) {}

  static from(fields: FieldsOf<GasUsed>) {
    return new GasUsed(fields.daGas, fields.l1Gas, fields.l2Gas);
  }

  static empty() {
    return new GasUsed(0, 0, 0);
  }

  isEmpty() {
    return this.daGas === 0 && this.l1Gas === 0 && this.l2Gas === 0;
  }

  static fromBuffer(buffer: Buffer | BufferReader): GasUsed {
    const reader = BufferReader.asReader(buffer);
    return new GasUsed(reader.readNumber(), reader.readNumber(), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.daGas, this.l1Gas, this.l2Gas);
  }

  [inspect.custom]() {
    return `GasUsed { daGas=${this.daGas} l1Gas=${this.l1Gas} l2Gas=${this.l2Gas} }`;
  }
}
