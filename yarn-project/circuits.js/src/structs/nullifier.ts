import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { type Ordered } from '../interfaces/index.js';

export class Nullifier implements Ordered {
  constructor(public value: Fr, public counter: number, public noteHash: Fr) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), this.noteHash];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new Nullifier(reader.readField(), reader.readU32(), reader.readField());
  }

  isEmpty() {
    return this.value.isZero() && !this.counter && this.noteHash.isZero();
  }

  static empty() {
    return new Nullifier(Fr.zero(), 0, Fr.zero());
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.noteHash);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new Nullifier(Fr.fromBuffer(reader), reader.readNumber(), Fr.fromBuffer(reader));
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter} noteHash=${this.noteHash}`;
  }

  scope(contractAddress: AztecAddress) {
    return new ScopedNullifier(this, contractAddress);
  }
}

export class ScopedNullifier implements Ordered {
  constructor(public nullifier: Nullifier, public contractAddress: AztecAddress) {}

  get counter() {
    return this.nullifier.counter;
  }

  get value() {
    return this.nullifier.value;
  }

  get nullifiedNoteHash() {
    return this.nullifier.noteHash;
  }

  toFields(): Fr[] {
    return [...this.nullifier.toFields(), this.contractAddress.toField()];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ScopedNullifier(reader.readObject(Nullifier), AztecAddress.fromField(reader.readField()));
  }

  isEmpty() {
    return this.nullifier.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedNullifier(Nullifier.empty(), AztecAddress.ZERO);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.nullifier, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedNullifier(Nullifier.fromBuffer(reader), AztecAddress.fromBuffer(reader));
  }

  toString(): string {
    return `nullifier=${this.nullifier} contractAddress=${this.contractAddress}`;
  }
}
