import { Fq, type Fr } from '../fields/fields.js';
import { type Tuple } from './types.js';

/**
 * The FieldReader class provides a utility for reading various data types from a field array.
 *
 * Usage:
 * Create a new instance of FieldReader with an array of fields and an optional offset.
 * Use the provided methods to read desired data types from the field array.
 * The reading methods automatically advance the internal index.
 */
export class FieldReader {
  private index: number;
  private length: number;
  constructor(private fields: Fr[], offset = 0) {
    this.index = offset;
    this.length = fields.length;
    if (offset >= this.length) {
      throw new Error('Offset out of bounds.');
    }
  }

  /**
   * Creates a FieldReader instance from either a field array or an existing FieldReader.
   *
   * @param fields - A field array or FieldReader to initialize the FieldReader.
   * @returns An instance of FieldReader.
   */
  public static asReader(fields: Fr[] | FieldReader): FieldReader {
    if (fields instanceof FieldReader) {
      return fields;
    }

    return new FieldReader(fields);
  }

  /**
   * Reads a single field from the array.
   *
   * @returns A field.
   */
  public readField(): Fr {
    if (this.index === this.length) {
      throw new Error('Not enough fields to be consumed.');
    }
    return this.fields[this.index++];
  }

  /**
   * Reads a Fq from the array.
   *
   * @returns An Fq.
   */
  public readFq(): Fq {
    return Fq.fromHighLow(this.readField(), this.readField());
  }

  /**
   * Reads and returns the next boolean value from the field array.
   * Advances the internal index by 1, treating the field at the current index as a boolean value.
   * Returns true if the field is non-zero, false otherwise.
   * Throw if the value is not 0 or 1.
   *
   * @returns A boolean value representing the field at the current index.
   */
  public readBoolean(): boolean {
    const field = this.readField();
    const value = field.toBigInt();
    if (value > 1n) {
      throw new Error('Field is not a boolean.');
    }
    return value == 1n;
  }

  /**
   * Reads a 32-bit unsigned integer from the field array at the current index position.
   * Updates the index position by 1 after reading the number.
   * Throw if the value is greater than 2 ** 32.
   *
   * @returns The read 32-bit unsigned integer value.
   */
  public readU32(): number {
    const field = this.readField();
    const value = field.toBigInt();
    if (value >= 1n << 32n) {
      throw new Error('Field is not a u32.');
    }
    return Number(value);
  }

  /**
   * Read an array of a fixed size field array.
   *
   * @param size - The fixed number of fields in the array.
   * @returns An array of fields.
   */
  public readFieldArray<N extends number>(size: N): Tuple<Fr, N> {
    const result: Fr[] = [];
    for (let i = 0; i < size; ++i) {
      result.push(this.readField());
    }
    return result as Tuple<Fr, N>;
  }

  /**
   * Read an array of a fixed size with elements of type T from the field array.
   * The 'itemDeserializer' object should have a 'fromFields' method that takes a FieldReader instance as input,
   * and returns an instance of the desired deserialized data type T.
   * This method will call the 'fromFields' method for each element in the array and return the resulting array.
   *
   * @param size - The fixed number of elements in the array.
   * @param itemDeserializer - An object with a 'fromFields' method to deserialize individual elements of type T.
   * @returns An array of instances of type T.
   */
  public readArray<T, N extends number>(
    size: N,
    itemDeserializer: {
      /**
       * A function for deserializing data from a FieldReader instance.
       */
      fromFields: (reader: FieldReader) => T;
    },
  ): Tuple<T, N> {
    const result = Array.from({ length: size }, () => itemDeserializer.fromFields(this));
    return result as Tuple<T, N>;
  }

  /**
   * Reads a serialized object from a field array and returns the deserialized object using the given deserializer.
   *
   * @typeparam T - The type of the deserialized object.
   * @param deserializer - An object with a 'fromFields' method that takes a FieldReader instance and returns an instance of the deserialized object.
   * @returns The deserialized object of type T.
   */
  public readObject<T>(deserializer: {
    /**
     * A method that takes a FieldReader instance and returns an instance of the deserialized data type.
     */
    fromFields: (reader: FieldReader) => T;
  }): T {
    return deserializer.fromFields(this);
  }
}
