import { BufferReader } from '../serialize/index.js';

/**
 * Holds an opaque pointer into WASM memory.
 * Currently only 4 bytes, but could grow to 8 bytes with wasm64.
 */
export class Ptr {
  static SIZE_IN_BYTES = 4;

  constructor(public readonly value: Uint8Array) {}

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  toBuffer() {
    return this.value;
  }
}
