import { inspect } from 'util';

import { Fr, fromBuffer } from '../fields/index.js';
import { type BufferReader, FieldReader } from '../serialize/index.js';
import { TypeRegistry } from '../serialize/type_registry.js';

/**
 * AztecAddress represents a 32-byte address in the Aztec Protocol.
 * It provides methods to create, manipulate, and compare addresses.
 * The maximum value of an address is determined by the field modulus and all instances of AztecAddress.
 * It should have a value less than or equal to this max value.
 * This class also provides helper functions to convert addresses from strings, buffers, and other formats.
 */
export class AztecAddress extends Fr {
  constructor(buffer: Buffer) {
    if (buffer.length !== 32) {
      throw new Error(`Invalid AztecAddress length ${buffer.length}.`);
    }
    super(buffer);
  }

  [inspect.custom]() {
    return `AztecAddress<${this.toString()}>`;
  }

  static override ZERO = new AztecAddress(Buffer.alloc(32));

  static override zero(): AztecAddress {
    return AztecAddress.ZERO;
  }

  static override fromBuffer(buffer: Buffer | BufferReader) {
    return fromBuffer(buffer, AztecAddress);
  }

  static fromField(fr: Fr) {
    return new AztecAddress(fr.toBuffer());
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return AztecAddress.fromField(reader.readField());
  }

  static fromBigInt(value: bigint) {
    return AztecAddress.fromField(new Fr(value));
  }

  static override fromString(buf: string) {
    const buffer = Buffer.from(buf.replace(/^0x/i, ''), 'hex');
    return new AztecAddress(buffer);
  }

  static override random() {
    return new AztecAddress(super.random().toBuffer());
  }

  override toJSON() {
    return {
      type: 'AztecAddress',
      value: this.toString(),
    };
  }
}

// For deserializing JSON.
TypeRegistry.register('AztecAddress', AztecAddress);
