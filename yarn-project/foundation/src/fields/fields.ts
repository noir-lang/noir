import { BufferReader } from '../serialize/buffer_reader.js';
import { numToUInt32BE } from '../serialize/index.js';

abstract class Field {
  public static SIZE_IN_BYTES = 32;

  private buffer: Buffer;

  constructor(input: Buffer | number) {
    if (Buffer.isBuffer(input)) {
      if (input.length != Field.SIZE_IN_BYTES) {
        throw new Error(`Unexpected buffer size ${input.length} (expected ${Field.SIZE_IN_BYTES} bytes)`);
      }
      this.buffer = input;
    } else {
      if (BigInt(input) > this.maxValue()) {
        throw new Error(`Input value ${input} too large (expected ${this.maxValue()})`);
      }
      this.buffer = numToUInt32BE(input, 32);
    }
  }

  abstract maxValue(): bigint;

  toString() {
    return '0x' + this.buffer.toString('hex');
  }

  toBuffer() {
    return this.buffer;
  }
}

export class Fr extends Field {
  /**
   * Maximum represntable value in a field is the curve prime minus one.
   * @returns 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000n
   */
  maxValue() {
    return 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n - 1n;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }
}

export class Fq extends Field {
  /**
   * Maximum represntable vaue in a field is the curve prime minus one.
   * TODO: Find out actual max value for Fq.
   * @returns 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000n
   */
  maxValue() {
    return 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n - 1n;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }
}
