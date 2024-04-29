import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

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

  toString(): string {
    return `value=${this.value.toString()} counter=${this.counter.toString()}`;
  }

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
    return SideEffect.isEmpty(this);
  }

  /**
   * Checks whether this instance of side-effect is empty.
   * @returns True if the value and counter both are zero.
   */
  static isEmpty(sideEffect: SideEffect) {
    return sideEffect.value.isZero() && sideEffect.counter.isZero();
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
