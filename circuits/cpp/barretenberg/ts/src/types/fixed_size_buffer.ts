import { randomBytes } from '../random/index.js';
import { BufferReader } from '../serialize/index.js';

export class Buffer32 {
  static SIZE_IN_BYTES = 32;

  constructor(public readonly buffer: Uint8Array) {}

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new Buffer32(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static random() {
    return new Buffer32(randomBytes(this.SIZE_IN_BYTES));
  }

  toBuffer() {
    return this.buffer;
  }
}

export class Buffer64 {
  static SIZE_IN_BYTES = 64;

  constructor(public readonly buffer: Uint8Array) {}

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new Buffer64(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static random() {
    return new Buffer64(randomBytes(this.SIZE_IN_BYTES));
  }

  toBuffer() {
    return this.buffer;
  }
}

export class Buffer128 {
  static SIZE_IN_BYTES = 128;

  constructor(public readonly buffer: Uint8Array) {}

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new Buffer128(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static random() {
    return new Buffer128(randomBytes(this.SIZE_IN_BYTES));
  }

  toBuffer() {
    return this.buffer;
  }
}
