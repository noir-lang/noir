import {
  deserializeArrayFromVector,
  deserializeBigInt,
  deserializeBool,
  deserializeBufferFromVector,
  deserializeInt32,
  deserializeUInt32,
} from './free_funcs.js';

/**
 * DeserializeFn is a type representing a deserialization function for a specific data type. The function takes
 * a buffer and an offset as input, and returns an object containing the deserialized element of the data type and
 * the number of bytes advanced in the buffer. This type is used to provide custom deserialization logic for arrays,
 * objects or custom data types while working with the Deserializer class.
 */
export type DeserializeFn<T> = (
  buf: Buffer,
  offset: number,
) => {
  /**
   * The deserialized element of the specified data type.
   */
  elem: T;
  /**
   * The number of bytes advanced in the buffer during deserialization.
   */
  adv: number;
};

/**
 * Deserializer class provides a set of methods to deserialize different data types from a buffer.
 * It maintains an internal buffer and offset, updating the offset as it deserializes each data type.
 * The class supports deserialization of various data types including boolean, integers, big integers,
 * buffers, strings, dates, and arrays with custom deserialization functions.
 *
 * @example
 * const deserializer = new Deserializer(buffer);
 * const boolValue = deserializer.bool();
 * const intValue = deserializer.int32();
 * const bigIntValue = deserializer.bigInt();
 * const stringValue = deserializer.string();
 * const dateValue = deserializer.date();
 * const arrayValue = deserializer.deserializeArray(customDeserializeFn);
 */
export class Deserializer {
  constructor(private buf: Buffer, private offset = 0) {}

  /**
   * Deserialize a boolean value from the buffer at the current offset.
   * Advances the internal offset by one byte after deserialization.
   * Returns 'true' if the deserialized value is non-zero, otherwise returns 'false'.
   *
   * @returns The deserialized boolean value.
   */
  public bool() {
    return this.exec(deserializeBool) ? true : false;
  }

  /**
   * Deserialize a 32-bit unsigned integer from the buffer at the current offset.
   * Advances the internal buffer offset by 4 after successful deserialization.
   * The result is returned as a JavaScript number.
   *
   * @returns A 32-bit unsigned integer value.
   */
  public uInt32() {
    return this.exec(deserializeUInt32);
  }

  /**
   * Deserialize a 32-bit signed integer from the internal buffer.
   * Reads 4 bytes from the current offset in the buffer and interprets them as a little-endian int32 value.
   * Advances the internal offset by 4 bytes after successful deserialization.
   *
   * @returns The deserialized 32-bit signed integer value.
   */
  public int32() {
    return this.exec(deserializeInt32);
  }

  /**
   * Deserialize a BigInt from the buffer, taking into account the specified width.
   * The method reads 'width' bytes from the buffer starting at the current offset and converts it to a BigInt.
   * The offset is advanced by 'width' bytes after successful deserialization.
   *
   * @param width - The number of bytes to read from the buffer to construct the BigInt (default is 32).
   * @returns The deserialized BigInt value.
   */
  public bigInt(width = 32) {
    return this.exec((buf: Buffer, offset: number) => deserializeBigInt(buf, offset, width));
  }

  /**
   * Deserialize a variable-length byte array from the internal buffer.
   * This method reads the length of the array and then extracts the corresponding bytes.
   * It advances the internal offset by the number of bytes read, including the length prefix.
   *
   * @returns A Buffer instance containing the deserialized byte array.
   */
  public vector() {
    return this.exec(deserializeBufferFromVector);
  }

  /**
   * Extract a sub-buffer with the specified width, advancing the internal offset.
   * The function slices the buffer from the current offset to the offset plus the provided width,
   * and advances the internal offset by the width. This can be useful for working with fixed-width
   * structures within the original buffer.
   *
   * @param width - The number of bytes to include in the extracted sub-buffer.
   * @returns A sub-buffer containing the specified number of bytes from the original buffer.
   */
  public buffer(width: number) {
    const buf = this.buf.slice(this.offset, this.offset + width);
    this.offset += width;
    return buf;
  }

  /**
   * Deserialize a string from the internal buffer.
   * It first deserializes a vector representing the UTF-8 encoded string from the buffer,
   * and then converts it to a string.
   *
   * @returns The deserialized string.
   */
  public string() {
    return this.vector().toString();
  }

  /**
   * Deserialize a Date object from the internal buffer.
   * The date value is expected to be stored as a 64-bit BigInt representing the number of milliseconds since the Unix epoch.
   * Advances the internal offset by 8 bytes after deserialization.
   *
   * @returns A Date instance representing the deserialized date value.
   */
  public date() {
    return new Date(Number(this.bigInt(8)));
  }

  /**
   * Deserialize an array of elements using the provided deserialization function.
   * This method reads the serialized data from the buffer and deserializes each element in the array
   * using the given 'fn' deserialization function. The returned array contains the deserialized elements
   * in their original order.
   *
   * @param fn - The deserialization function to be applied on each element in the array.
   * @returns An array containing the deserialized elements.
   */
  public deserializeArray<T>(fn: DeserializeFn<T>) {
    return this.exec((buf: Buffer, offset: number) => deserializeArrayFromVector(fn, buf, offset));
  }

  /**
   * Executes the given deserialization function on this Deserializer's buffer and updates the internal offset.
   * The DeserializeFn<T> should take a Buffer and an offset as input, and return an object containing the deserialized
   * element and the number of bytes advanced in the buffer. This method is useful for custom deserialization logic
   * or implementing new deserialization functions.
   *
   * @typeparam T - The type of the deserialized element.
   * @param fn - The deserialization function to execute.
   * @returns The deserialized element of type T.
   */
  public exec<T>(fn: DeserializeFn<T>): T {
    const { elem, adv } = fn(this.buf, this.offset);
    this.offset += adv;
    return elem;
  }

  /**
   * Returns the current offset value in the Deserializer instance.
   * The offset is updated as elements are deserialized from the buffer.
   * It can be useful for tracking the position in the buffer during complex deserialization processes.
   *
   * @returns The current offset value as a number.
   */
  public getOffset() {
    return this.offset;
  }
}
