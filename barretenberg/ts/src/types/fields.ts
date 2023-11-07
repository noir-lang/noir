import { randomBytes } from '../random/index.js';
import { toBigIntBE, toBufferBE } from '../bigint-array/index.js';
import { BufferReader, uint8ArrayToHexString } from '../serialize/index.js';

export class Fr {
  static ZERO = new Fr(0n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = this.MODULUS - 1n;
  static SIZE_IN_BYTES = 32;
  value: Uint8Array;

  constructor(value: Uint8Array | bigint) {
    if (typeof value === 'bigint') {
      if (value > Fr.MAX_VALUE) {
        throw new Error(`Fr out of range ${value}.`);
      }
      this.value = toBufferBE(value);
    } else {
      this.value = value;
    }
  }

  static random() {
    const r = toBigIntBE(randomBytes(64)) % Fr.MODULUS;
    return new this(r);
  }

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static fromBufferReduce(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(toBigIntBE(reader.readBytes(this.SIZE_IN_BYTES)) % Fr.MODULUS);
  }

  static fromString(str: string) {
    return this.fromBuffer(Buffer.from(str.replace(/^0x/i, ''), 'hex'));
  }

  toBuffer() {
    return this.value;
  }

  toString() {
    return '0x' + uint8ArrayToHexString(this.toBuffer());
  }

  equals(rhs: Fr) {
    return this.value.every((v, i) => v === rhs.value[i]);
  }

  isZero() {
    return this.value.every(v => v === 0);
  }
}

export class Fq {
  static MODULUS = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47n;
  static MAX_VALUE = this.MODULUS - 1n;
  static SIZE_IN_BYTES = 32;

  constructor(public readonly value: bigint) {
    if (value > Fq.MAX_VALUE) {
      throw new Error(`Fq out of range ${value}.`);
    }
  }

  static random() {
    const r = toBigIntBE(randomBytes(64)) % Fq.MODULUS;
    return new this(r);
  }

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(toBigIntBE(reader.readBytes(this.SIZE_IN_BYTES)));
  }

  static fromBufferReduce(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(toBigIntBE(reader.readBytes(this.SIZE_IN_BYTES)) % Fr.MODULUS);
  }

  static fromString(str: string) {
    return this.fromBuffer(Buffer.from(str.replace(/^0x/i, ''), 'hex'));
  }

  toBuffer() {
    return toBufferBE(this.value, Fq.SIZE_IN_BYTES);
  }

  toString() {
    return '0x' + this.value.toString(16);
  }

  equals(rhs: Fq) {
    return this.value === rhs.value;
  }

  isZero() {
    return this.value === 0n;
  }
}
