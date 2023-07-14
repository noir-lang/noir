import {
  boolToByte,
  numToInt32BE,
  numToUInt32BE,
  serializeBigInt,
  serializeBufferToVector,
  serializeDate,
} from './free_funcs.js';
import { serializeBufferArrayToVector } from './index.js';

/**
 * The Serializer class provides a convenient and efficient way to serialize various data types into binary format.
 * It supports serialization of primitive types (like boolean, signed/unsigned integers, BigInt), as well as more complex types (like Buffer, Date, and custom structures with their own 'toBuffer()' methods).
 * The class maintains an internal buffer array that accumulates serialized data, allowing for easy concatenation and retrieval of the final serialized Buffer.
 * This can be useful in various applications such as network communication, file storage, or other scenarios where binary data representation is needed.
 */
export class Serializer {
  private buf: Buffer[] = [];

  constructor() {}

  /**
   * Serialize a boolean value into a Buffer and append it to the internal buffer array.
   * The serialized boolean will be stored as a single byte, where true is represented as 1 and false as 0.
   * This method updates the Serializer's internal state and does not return any values.
   *
   * @param bool - The boolean value to be serialized.
   */
  public bool(bool: boolean) {
    this.buf.push(boolToByte(bool));
  }

  /**
   * Encodes a given unsigned 32-bit integer into a Buffer and appends it to the internal buffer array.
   * The provided number should be within the range of 0 and 2^32 - 1, inclusive.
   * Throws an error if the input value is out of range or not a valid number.
   *
   * @param num - The unsigned 32-bit integer to be encoded and appended.
   */
  public uInt32(num: number) {
    this.buf.push(numToUInt32BE(num));
  }

  /**
   * Serialize a signed 32-bit integer (int32) into the internal buffer.
   * The number should be within the range of -2147483648 to 2147483647 inclusive.
   * Throws an error if the input is not within the valid int32 range.
   *
   * @param num - The signed 32-bit integer to serialize.
   */
  public int32(num: number) {
    this.buf.push(numToInt32BE(num));
  }

  /**
   * Serializes a BigInt into a Buffer and appends it to the internal buffer array.
   * The given 'num' is treated as a signed integer and is serialized using
   * little-endian byte order. This method is useful for efficiently storing
   * large integer values that may not fit within the range of a standard number.
   *
   * @param num - The BigInt value to serialize.
   */
  public bigInt(num: bigint) {
    this.buf.push(serializeBigInt(num));
  }

  /**
   * The given buffer is of variable length. Prefixes the buffer with its length.
   * @param buf - The buffer to serialize as a variable-length vector.
   */
  public vector(buf: Buffer) {
    this.buf.push(serializeBufferToVector(buf));
  }

  /**
   * Directly serializes a buffer that maybe of fixed, or variable length.
   * It is assumed the corresponding deserialize function will handle variable length data, thus the length
   * does not need to be prefixed here.
   * If serializing a raw, variable length buffer, use vector().
   * @param buf - The buffer to serialize as a fixed-length array.
   */
  public buffer(buf: Buffer) {
    this.buf.push(buf);
  }

  /**
   * Serialize a string by first converting it to a buffer and then encoding its length as a prefix.
   * The serialized string can be deserialized by reading the prefixed length and extracting the corresponding data.
   * This method is useful for serializing strings of variable length in a consistent format.
   *
   * @param str - The input string to be serialized.
   */
  public string(str: string) {
    this.vector(Buffer.from(str));
  }

  /**
   * Serialize a given Date instance into a Buffer and append it to the internal buffer list.
   * The serialized date is stored as an 8-byte BigInt representing the number of milliseconds since the Unix epoch.
   * This function facilitates serialization of JavaScript's built-in Date objects for subsequent data transmission or storage.
   *
   * @param date - The Date instance to be serialized.
   */
  public date(date: Date) {
    this.buf.push(serializeDate(date));
  }

  /**
   * Returns the serialized Buffer object that was created by calling various serialization methods on this Serializer instance.
   * The resulting buffer can be used for sending or storing serialized data in binary format.
   *
   * @returns A Buffer containing the serialized data.
   */
  public getBuffer() {
    return Buffer.concat(this.buf);
  }

  /**
   * Serializes an array of elements, where each element has a 'toBuffer()' method, into a single Buffer.
   * The resulting buffer is prefixed with its length (number of elements), allowing for easy deserialization.
   * This method is useful for serializing arrays of custom classes or data structures that have their own serialization logic.
   *
   * @param arr - The array of elements to be serialized. Each element must have a 'toBuffer()' method for serialization.
   */
  public serializeArray<T>(arr: T[]) {
    this.buf.push(serializeBufferArrayToVector(arr.map((e: any) => e.toBuffer())));
  }
}
