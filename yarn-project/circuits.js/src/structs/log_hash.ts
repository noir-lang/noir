import { AztecAddress } from '@aztec/foundation/aztec-address';
import { sha256Trunc } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import { type Ordered } from '../interfaces/index.js';

export class LogHash implements Ordered {
  constructor(public value: Fr, public counter: number, public length: Fr) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), this.length];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new LogHash(reader.readField(), reader.readU32(), reader.readField());
  }

  isEmpty() {
    return this.value.isZero() && this.length.isZero() && !this.counter;
  }

  static empty() {
    return new LogHash(Fr.zero(), 0, Fr.zero());
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.length);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new LogHash(Fr.fromBuffer(reader), reader.readNumber(), Fr.fromBuffer(reader));
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter} length=${this.length}`;
  }

  [inspect.custom](): string {
    return `LogHash { ${this.toString()} }`;
  }
}

export class ScopedLogHash implements Ordered {
  constructor(public logHash: LogHash, public contractAddress: AztecAddress) {}

  get counter() {
    return this.logHash.counter;
  }

  get value() {
    return this.logHash.value;
  }

  toFields(): Fr[] {
    return [...this.logHash.toFields(), this.contractAddress.toField()];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ScopedLogHash(reader.readObject(LogHash), AztecAddress.fromField(reader.readField()));
  }

  isEmpty() {
    return this.logHash.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedLogHash(LogHash.empty(), AztecAddress.ZERO);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.logHash, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedLogHash(LogHash.fromBuffer(reader), AztecAddress.fromBuffer(reader));
  }

  toString(): string {
    return `logHash=${this.logHash} contractAddress=${this.contractAddress}`;
  }

  getSiloedHash(): Buffer {
    return sha256Trunc(Buffer.concat([this.contractAddress.toBuffer(), this.value.toBuffer()]));
  }
}

export class NoteLogHash implements Ordered {
  constructor(public value: Fr, public counter: number, public length: Fr, public noteHashCounter: number) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), this.length, new Fr(this.noteHashCounter)];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new NoteLogHash(reader.readField(), reader.readU32(), reader.readField(), reader.readU32());
  }

  isEmpty() {
    return this.value.isZero() && this.length.isZero() && !this.counter && !this.noteHashCounter;
  }

  static empty() {
    return new NoteLogHash(Fr.zero(), 0, Fr.zero(), 0);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.length, this.noteHashCounter);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NoteLogHash(Fr.fromBuffer(reader), reader.readNumber(), Fr.fromBuffer(reader), reader.readNumber());
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter} length=${this.length} noteHashCounter=${this.noteHashCounter}`;
  }
}

export class EncryptedLogHash implements Ordered {
  constructor(public value: Fr, public counter: number, public length: Fr, public randomness: Fr) {}

  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), this.length, this.randomness];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new EncryptedLogHash(reader.readField(), reader.readU32(), reader.readField(), reader.readField());
  }

  isEmpty() {
    return this.value.isZero() && this.length.isZero() && !this.counter && this.randomness.isZero();
  }

  static empty() {
    return new EncryptedLogHash(Fr.zero(), 0, Fr.zero(), Fr.zero());
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.length, this.randomness);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new EncryptedLogHash(
      Fr.fromBuffer(reader),
      reader.readNumber(),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  toString(): string {
    return `value=${this.value} counter=${this.counter} length=${this.length} randomness=${this.randomness}`;
  }
}

export class ScopedEncryptedLogHash implements Ordered {
  constructor(public logHash: EncryptedLogHash, public contractAddress: AztecAddress) {}

  get counter() {
    return this.logHash.counter;
  }

  get value() {
    return this.logHash.value;
  }

  toFields(): Fr[] {
    return [...this.logHash.toFields(), this.contractAddress.toField()];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ScopedEncryptedLogHash(reader.readObject(EncryptedLogHash), AztecAddress.fromField(reader.readField()));
  }

  isEmpty() {
    return this.logHash.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedEncryptedLogHash(EncryptedLogHash.empty(), AztecAddress.ZERO);
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.logHash, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedEncryptedLogHash(EncryptedLogHash.fromBuffer(reader), AztecAddress.fromBuffer(reader));
  }

  toString(): string {
    return `logHash=${this.logHash} contractAddress=${this.contractAddress}`;
  }
}
