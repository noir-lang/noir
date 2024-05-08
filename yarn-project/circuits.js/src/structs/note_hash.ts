import { AztecAddress } from '@aztec/foundation/aztec-address';
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

  scope(nullifierCounter: number, contractAddress: AztecAddress) {
    return new ScopedNoteHash(this, nullifierCounter, contractAddress);
  }
}

export class ScopedNoteHash implements Ordered {
  constructor(public noteHash: NoteHash, public nullifierCounter: number, public contractAddress: AztecAddress) {}

  get counter() {
    return this.noteHash.counter;
  }

  get value() {
    return this.noteHash.value;
  }

  toFields(): Fr[] {
    return [...this.noteHash.toFields(), new Fr(this.nullifierCounter), this.contractAddress.toField()];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ScopedNoteHash(
      reader.readObject(NoteHash),
      reader.readU32(),
      AztecAddress.fromField(reader.readField()),
    );
  }

  isEmpty() {
    return this.noteHash.isEmpty() && !this.nullifierCounter && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedNoteHash(NoteHash.empty(), 0, AztecAddress.ZERO);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.noteHash, this.nullifierCounter, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedNoteHash(NoteHash.fromBuffer(reader), reader.readNumber(), AztecAddress.fromBuffer(reader));
  }

  toString(): string {
    return `noteHash=${this.noteHash} nullifierCounter=${this.nullifierCounter} contractAddress=${this.contractAddress}`;
  }
}
