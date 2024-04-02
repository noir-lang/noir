import { fromHex, toBigIntBE } from '../bigint-buffer/index.js';
import { keccak, randomBytes } from '../crypto/index.js';
import { type Fr } from '../fields/fields.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { Selector } from './selector.js';

/* eslint-disable @typescript-eslint/no-unsafe-declaration-merging */

/** Event selector branding */
export interface EventSelector {
  /** Brand. */
  _branding: 'EventSelector';
}

/** An event selector is the first 4 bytes of the hash of an event signature. */
export class EventSelector extends Selector {
  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer  or BufferReader to read from.
   * @returns The Selector.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const value = Number(toBigIntBE(reader.readBytes(Selector.SIZE)));
    return new EventSelector(value);
  }

  /**
   * Converts a field to selector.
   * @param fr - The field to convert.
   * @returns The selector.
   */
  static fromField(fr: Fr) {
    return new EventSelector(Number(fr.toBigInt()));
  }

  /**
   * Creates a selector from a signature.
   * @param signature - Signature to generate the selector for (e.g. "transfer(field,field)").
   * @returns selector.
   */
  static fromSignature(signature: string) {
    // throw if signature contains whitespace
    if (/\s/.test(signature)) {
      throw new Error('Signature cannot contain whitespace');
    }
    return EventSelector.fromBuffer(keccak(Buffer.from(signature)).subarray(0, Selector.SIZE));
  }

  /**
   * Create a Selector instance from a hex-encoded string.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 64 hex characters.
   * Throws an error if the input length is invalid or address value is out of range.
   *
   * @param selector - The hex-encoded string representing the Selector.
   * @returns An Selector instance.
   */
  static fromString(selector: string) {
    const buf = fromHex(selector);
    if (buf.length !== Selector.SIZE) {
      throw new Error(`Invalid Selector length ${buf.length} (expected ${Selector.SIZE}).`);
    }
    return EventSelector.fromBuffer(buf);
  }

  /**
   * Creates an empty selector.
   * @returns An empty selector.
   */
  static empty() {
    return new EventSelector(0);
  }

  /**
   * Creates a random selector.
   * @returns A random selector.
   */
  static random() {
    return EventSelector.fromBuffer(randomBytes(Selector.SIZE));
  }
}
