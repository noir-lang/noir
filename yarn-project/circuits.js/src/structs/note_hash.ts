import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { type Ordered } from '../interfaces/index.js';

export class NoteHash {
  constructor(public value: Fr, public counter: number) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter)];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new NoteHash(reader.readField(), reader.readU32());
  }

  isEmpty() {
    return this.value.isZero() && !this.counter;
  }

  static empty() {
    return new NoteHash(Fr.zero(), 0);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NoteHash(Fr.fromBuffer(reader), reader.readNumber());
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter}`;
  }
}

export class NoteHashContext implements Ordered {
  constructor(public value: Fr, public counter: number, public nullifierCounter: number) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), new Fr(this.nullifierCounter)];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new NoteHashContext(reader.readField(), reader.readU32(), reader.readU32());
  }

  isEmpty() {
    return this.value.isZero() && !this.counter && !this.nullifierCounter;
  }

  static empty() {
    return new NoteHashContext(Fr.zero(), 0, 0);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.nullifierCounter);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NoteHashContext(Fr.fromBuffer(reader), reader.readNumber(), reader.readNumber());
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter} nullifierCounter=${this.nullifierCounter}`;
  }
}
