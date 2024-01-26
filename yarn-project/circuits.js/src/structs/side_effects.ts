import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Fr } from './index.js';

/**
 * Essential members and functions of all SideEffect variants
 */
export interface SideEffectType {
  /** The actual value associated with the SideEffect */
  value: Fr;
  /** The counter associated with the SideEffect */
  counter: Fr;
  /** Convert to a buffer */
  toBuffer(): Buffer;
  /** Convert to a field array */
  toFields(): Fr[];
  /** Are all of the fields of the SideEffect zero? */
  isEmpty(): boolean;
}

/**
 * Side-effect object consisting of a value and a counter.
 * cpp/src/aztec3/circuits/abis/side_effects.hpp.
 */
export class SideEffect implements SideEffectType {
  constructor(
    /**
     * The value of the side-effect object.
     */
    public value: Fr,
    /**
     * The side-effect counter.
     */
    public counter: Fr,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter);
  }

  /**
   * Convert to an array of fields.
   * @returns The array of fields.
   */
  toFields(): Fr[] {
    return [this.value, this.counter];
  }

  static fromFields(fields: Fr[] | FieldReader): SideEffect {
    const reader = FieldReader.asReader(fields);
    return new SideEffect(reader.readField(), reader.readField());
  }

  /**
   * Returns whether this instance of side-effect is empty.
   * @returns True if the value and counter both are zero.
   */
  isEmpty() {
    return this.value.isZero() && this.counter.isZero();
  }

  /**
   * Returns an empty instance of side-effect.
   * @returns Side-effect with both value and counter being zero.
   */
  static empty(): SideEffect {
    return new SideEffect(Fr.zero(), Fr.zero());
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of SideEffect.
   */
  static fromBuffer(buffer: Buffer | BufferReader): SideEffect {
    const reader = BufferReader.asReader(buffer);
    return new SideEffect(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }
}

/**
 * Side-effect object consisting of a value, a start counter and an end counter.
 * cpp/src/aztec3/circuits/abis/side_effects.hpp.
 */
export class SideEffectLinkedToNoteHash implements SideEffectType {
  constructor(
    /**
     * The value of the side-effect object.
     */
    public value: Fr,
    /**
     * The note hash corresponding to the side-effect value.
     */
    public noteHash: Fr,
    /**
     * The counter.
     */
    public counter: Fr,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.noteHash, this.counter);
  }

  /**
   * Convert to an array of fields.
   * @returns The array of fields.
   */
  toFields(): Fr[] {
    return [this.value, this.noteHash, this.counter];
  }

  static fromFields(fields: Fr[] | FieldReader): SideEffectLinkedToNoteHash {
    const reader = FieldReader.asReader(fields);
    return new SideEffectLinkedToNoteHash(reader.readField(), reader.readField(), reader.readField());
  }

  /**
   * Returns whether this instance of side-effect is empty.
   * @returns True if the value, note hash and counter are all zero.
   */
  isEmpty() {
    return this.value.isZero() && this.noteHash.isZero() && this.counter.isZero();
  }

  /**
   * Returns an empty instance of side-effect.
   * @returns Side-effect with value, note hash and counter being zero.
   */
  static empty(): SideEffectLinkedToNoteHash {
    return new SideEffectLinkedToNoteHash(Fr.zero(), Fr.zero(), Fr.zero());
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of SideEffectLinkedToNoteHash.
   */
  static fromBuffer(buffer: Buffer | BufferReader): SideEffectLinkedToNoteHash {
    const reader = BufferReader.asReader(buffer);
    return new SideEffectLinkedToNoteHash(Fr.fromBuffer(reader), Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }
}

/**
 * Convert an array of side effects to an array only non-empty side effects.
 * @param sideEffects - array to be converted
 * @returns the array of the non-empty side effects
 */
export function nonEmptySideEffects(sideEffects: SideEffectType[]): SideEffectType[] {
  return sideEffects.filter!(sideEffect => !sideEffect.isEmpty());
}

/**
 * Convert an array of side effects to an array of their values.
 * @param sideEffects - array to be converted
 * @returns the array of field values (excluding SideEffect metadata like counter)
 */
export function sideEffectArrayToValueArray(sideEffects: SideEffectType[]): Fr[] {
  return sideEffects.map(sideEffect => sideEffect.value);
}
