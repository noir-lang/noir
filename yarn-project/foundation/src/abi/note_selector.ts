import { toBigIntBE } from '../bigint-buffer/index.js';
import { randomBytes } from '../crypto/index.js';
import { type Fr } from '../fields/fields.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { TypeRegistry } from '../serialize/type_registry.js';
import { Selector } from './selector.js';

/* eslint-disable @typescript-eslint/no-unsafe-declaration-merging */

/** Note selector branding */
export interface NoteSelector {
  /** Brand. */
  _branding: 'NoteSelector';
}

/** A note selector is the first 4 bytes of the hash of a note signature. */
export class NoteSelector extends Selector {
  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer  or BufferReader to read from.
   * @returns The Selector.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const value = Number(toBigIntBE(reader.readBytes(Selector.SIZE)));
    return new NoteSelector(value);
  }

  static fromString(buf: string) {
    const withoutPrefix = buf.replace(/^0x/i, '');
    const buffer = Buffer.from(withoutPrefix, 'hex');
    return NoteSelector.fromBuffer(buffer);
  }

  /**
   * Converts a field to selector.
   * @param fr - The field to convert.
   * @returns The selector.
   */
  static fromField(fr: Fr) {
    return new NoteSelector(Number(fr.toBigInt()));
  }

  /**
   * Creates an empty selector.
   * @returns An empty selector.
   */
  static empty() {
    return new NoteSelector(0);
  }

  /**
   * Creates a random selector.
   * @returns A random selector.
   */
  static random() {
    return NoteSelector.fromBuffer(randomBytes(Selector.SIZE));
  }

  toJSON() {
    return {
      type: 'NoteSelector',
      value: this.toString(),
    };
  }

  static fromJSON(json: any): NoteSelector {
    return NoteSelector.fromString(json.value);
  }
}

// For deserializing JSON.
TypeRegistry.register('NoteSelector', NoteSelector);
