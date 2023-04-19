import { serializeBufferArrayToVector } from './index.js';
import {
  boolToByte,
  numToInt32BE,
  numToUInt32BE,
  serializeBigInt,
  serializeBufferToVector,
  serializeDate,
} from './free_funcs.js';

// export type DeserializeFn<T> = (buf: Buffer, offset: number) => { elem: T; adv: number };

/**
 * The Serializer class is a utility for converting various data types into binary format (Buffer) suitable for transmission or storage.
 * It offers several methods to serialize different data types, such as bool, int32, uInt32, bigInt, vector, buffer, string, and date.
 * Additionally, it allows serializing arrays of elements with custom 'toBuffer' methods. Serialized data can be retrieved as a single
 * Buffer using the getBuffer method. The class ensures proper serialization of variable-length data by prefixing them with their corresponding length.
 */
export class Serializer {
  private buf: Buffer[] = [];

  constructor() {}

  /**
   * Serializes a boolean value into a Buffer and appends it to the internal buffer array.
   * The serialized byte can represent either true or false, using 1 for true and 0 for false.
   *
   * @param bool - The boolean value to be serialized.
   */
  public bool(bool: boolean) {
    this.buf.push(boolToByte(bool));
  }

  /**
   * Serialize an unsigned 32-bit integer into a big-endian byte Buffer and add it to the internal buffer list.
   * The input 'num' should be within the range of 0 to 2^32-1, inclusive.
   * Throws an error if the input value is out of range.
   *
   * @param num - The unsigned 32-bit integer to be serialized.
   */
  public uInt32(num: number) {
    this.buf.push(numToUInt32BE(num));
  }

  /**
   * Serializes the given signed 32-bit integer as a big-endian buffer and stores it in the internal buffer.
   * The input 'num' should be within the range of [-2147483648, 2147483647], inclusive.
   * Throws an error if the input value is out of range.
   *
   * @param num - The signed 32-bit integer to serialize.
   */
  public int32(num: number) {
    this.buf.push(numToInt32BE(num));
  }

  /**
   * Serialize a bigint value into a Buffer and append it to the internal buffer array.
   * The function takes care of handling large integer values that cannot be stored in
   * standard JavaScript number type, allowing serialization of big integers without loss of precision.
   *
   * @param num - The bigint value to be serialized and added to the buffer.
   */
  public bigInt(num: bigint) {
    this.buf.push(serializeBigInt(num));
  }

  /**
   * The given buffer is of variable length. Prefixes the buffer with its length.
   */
  public vector(buf: Buffer) {
    this.buf.push(serializeBufferToVector(buf));
  }

  /**
   * Directly serializes a buffer that maybe of fixed, or variable length.
   * It is assumed the corresponding deserialize function will handle variable length data, thus the length
   * does not need to be prefixed here.
   * If serializing a raw, variable length buffer, use vector().
   */
  public buffer(buf: Buffer) {
    this.buf.push(buf);
  }

  /**
   * Serialize a given string by first converting it to a Buffer and then appending its length as a prefix.
   * The converted buffer is pushed into the internal buffer array for further serialization.
   * This method ensures the corresponding deserialize function can correctly read variable length strings.
   *
   * @param str - The input string to be serialized.
   */
  public string(str: string) {
    this.vector(Buffer.from(str));
  }

  /**
   * Serializes a Date object and appends it to the buffer.
   * The date is converted into a 64-bit integer representing the number of milliseconds since
   * January 1, 1970, 00:00:00 UTC. This ensures accurate representation and reconstruction of dates
   * during serialization and deserialization processes.
   *
   * @param date - The Date object to be serialized.
   */
  public date(date: Date) {
    this.buf.push(serializeDate(date));
  }

  /**
   * Returns the serialized buffer obtained by concatenating all the serialized elements added to the Serializer instance.
   * The resulting buffer can be used for data transmission or storage, and can be deserialized later to retrieve the original elements.
   *
   * @returns A Buffer containing the serialized data from the Serializer instance.
   */
  public getBuffer() {
    return Buffer.concat(this.buf);
  }

  /**
   * Serializes an array of elements and appends it to the internal buffer as a vector.
   * Each element in the array is assumed to have a 'toBuffer' method which returns its serialized representation as a Buffer.
   * The serialized array is prefixed with its length, allowing for variable-length arrays to be deserialized correctly.
   *
   * @param arr - The array of elements to be serialized.
   */
  public serializeArray<T>(arr: T[]) {
    this.buf.push(serializeBufferArrayToVector(arr.map((e: any) => e.toBuffer())));
  }
}
