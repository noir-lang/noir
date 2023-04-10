import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/index.js';

/**
 * Convert a boolean value to its corresponding byte representation in a Buffer of size 1.
 * The function takes a boolean value and writes it into a new buffer as either 1 (true) or 0 (false).
 * This method is useful for converting a boolean value into a binary format that can be stored or transmitted easily.
 *
 * @param b - The boolean value to be converted.
 * @returns A Buffer containing the byte representation of the input boolean value.
 */
export function boolToByte(b: boolean) {
  const buf = Buffer.alloc(1);
  buf.writeUInt8(b ? 1 : 0);
  return buf;
}

/**
 * Convert a number into a 4-byte little-endian unsigned integer buffer.
 * The input number is serialized as an unsigned 32-bit integer in little-endian byte order,
 * and returned as a Buffer of specified size (defaults to 4).
 * If the provided bufferSize is greater than 4, the additional bytes will be padded with zeros.
 *
 * @param n - The number to be converted into a little-endian unsigned integer buffer.
 * @param bufferSize - Optional, the size of the output buffer (default value is 4).
 * @returns A Buffer containing the serialized little-endian unsigned integer representation of the input number.
 */
export function numToUInt32LE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32LE(n, bufferSize - 4);
  return buf;
}

/**
 * Convert a number to a big-endian unsigned 32-bit integer Buffer.
 * This function takes a number and an optional buffer size as input and creates a Buffer with the specified size (defaults to 4) containing the big-endian representation of the input number as an unsigned 32-bit integer. Note that the bufferSize should be greater than or equal to 4, otherwise the output Buffer might truncate the serialized value.
 *
 * @param n - The input number to be converted to a big-endian unsigned 32-bit integer Buffer.
 * @param bufferSize - Optional, the size of the output Buffer (default is 4).
 * @returns A Buffer containing the big-endian unsigned 32-bit integer representation of the input number.
 */
export function numToUInt32BE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32BE(n, bufferSize - 4);
  return buf;
}

/**
 * Serialize a number into a big-endian signed 32-bit integer Buffer with the specified buffer size.
 * This function converts the input number into its binary representation and stores it in a Buffer
 * with the provided buffer size. By default, the buffer size is set to 4 bytes which represents a 32-bit integer.
 * The function will use the last 4 bytes of the buffer to store the serialized number. If the input number
 * is outside the range of a 32-bit signed integer, the resulting serialization may be incorrect due to truncation.
 *
 * @param n - The number to be serialized as a signed 32-bit integer.
 * @param bufferSize - Optional, the size of the output Buffer (default is 4 bytes).
 * @returns A Buffer containing the serialized big-endian signed 32-bit integer.
 */
export function numToInt32BE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeInt32BE(n, bufferSize - 4);
  return buf;
}

/**
 * Convert a number to an 8-bit unsigned integer and return it as a Buffer of length 1.
 * The input number is written as an 8-bit unsigned integer into the buffer. This function
 * is useful for converting small numeric values to a standardized binary format that can be
 * easily stored or transmitted.
 *
 * @param n - The number to be converted to an 8-bit unsigned integer.
 * @returns A Buffer containing the 8-bit unsigned integer representation of the input number.
 */
export function numToUInt8(n: number) {
  const bufferSize = 1;
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt8(n, 0);
  return buf;
}

/**
 * Serialize a Buffer into a vector format by encoding the length of the buffer and concatenating it with the original buffer.
 * The resulting vector consists of a 4-byte header containing the big-endian representation of the original buffer's length, followed by the original buffer.
 * This function is useful when storing buffers as data structures with dynamic lengths and later deserializing them using 'deserializeBufferFromVector'.
 *
 * @param buf - The input Buffer to be serialized into a vector format.
 * @returns A Buffer containing the serialized vector with the encoded length header.
 */
export function serializeBufferToVector(buf: Buffer) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(buf.length, 0);
  return Buffer.concat([lengthBuf, buf]);
}

/**
 * Serialize a BigInt value into a Buffer of specified width.
 * The function converts the input BigInt into its big-endian representation and stores it in a Buffer of the given width.
 * If the width is not provided, a default value of 32 bytes will be used. It is important to provide an appropriate width
 * to avoid truncation or incorrect serialization of large BigInt values.
 *
 * @param n - The BigInt value to be serialized.
 * @param width - The width (in bytes) of the output Buffer, optional with default value 32.
 * @returns A Buffer containing the serialized BigInt value in big-endian format.
 */
export function serializeBigInt(n: bigint, width = 32) {
  return toBufferBE(n, width);
}

/**
 * Deserialize a big integer from a buffer, given an offset and width.
 * Reads the specified number of bytes from the buffer starting at the offset, converts it to a big integer, and returns the deserialized result along with the number of bytes read (advanced).
 *
 * @param buf - The buffer containing the big integer to be deserialized.
 * @param offset - The position in the buffer where the big integer starts. Defaults to 0.
 * @param width - The number of bytes to read from the buffer for the big integer. Defaults to 32.
 * @returns An object containing the deserialized big integer value ('elem') and the number of bytes advanced ('adv').
 */
export function deserializeBigInt(buf: Buffer, offset = 0, width = 32) {
  return { elem: toBigIntBE(buf.subarray(offset, offset + width)), adv: width };
}

/**
 * Serializes a Date object into a Buffer containing its timestamp as a big integer value.
 * The resulting Buffer has a fixed width of 8 bytes, representing a 64-bit big-endian integer.
 * This function is useful for converting date values into a binary format that can be stored or transmitted easily.
 *
 * @param date - The Date object to be serialized.
 * @returns A Buffer containing the serialized timestamp of the input Date object.
 */
export function serializeDate(date: Date) {
  return serializeBigInt(BigInt(date.getTime()), 8);
}

/**
 * Deserialize a buffer from a vector by reading the length from its first 4 bytes, and then extracting the contents of the buffer.
 * The function returns an object containing the deserialized buffer as 'elem' and the number of bytes advanced ('adv') after deserialization.
 *
 * @param vector - The input buffer containing the serialized vector.
 * @param offset - The starting position from where the deserialization should begin (default is 0).
 * @returns An object with the deserialized buffer as 'elem' and the number of bytes advanced ('adv') after deserialization.
 */
export function deserializeBufferFromVector(vector: Buffer, offset = 0) {
  const length = vector.readUInt32BE(offset);
  const adv = 4 + length;
  return { elem: vector.subarray(offset + 4, offset + adv), adv };
}

/**
 * Deserialize a boolean value from a given buffer at the specified offset.
 * Reads a single byte at the provided offset in the buffer and returns
 * the deserialized boolean value along with the number of bytes read (adv).
 *
 * @param buf - The buffer containing the serialized boolean value.
 * @param offset - The position in the buffer to start reading the boolean value.
 * @returns An object containing the deserialized boolean value (elem) and the number of bytes read (adv).
 */
export function deserializeBool(buf: Buffer, offset = 0) {
  const adv = 1;
  return { elem: buf.readUInt8(offset), adv };
}

/**
 * Deserialize a 4-byte unsigned integer from a buffer, starting at the specified offset.
 * The deserialization reads 4 bytes from the given buffer and converts it into a number.
 * Returns an object containing the deserialized unsigned integer and the number of bytes advanced (4).
 *
 * @param buf - The buffer containing the serialized unsigned integer.
 * @param offset - The starting position in the buffer to deserialize from (default is 0).
 * @returns An object with the deserialized unsigned integer as 'elem' and the number of bytes advanced ('adv') as 4.
 */
export function deserializeUInt32(buf: Buffer, offset = 0) {
  const adv = 4;
  return { elem: buf.readUInt32BE(offset), adv };
}

/**
 * Deserialize a signed 32-bit integer from a buffer at the given offset.
 * The input 'buf' should be a Buffer containing binary data, and 'offset' should be the position in the buffer
 * where the signed 32-bit integer starts. Returns an object with both the deserialized integer (elem) and the
 * number of bytes advanced in the buffer (adv, always equal to 4).
 *
 * @param buf - The buffer containing the binary data.
 * @param offset - Optional, the position in the buffer where the signed 32-bit integer starts (default is 0).
 * @returns An object with the deserialized integer as 'elem' and the number of bytes advanced as 'adv'.
 */
export function deserializeInt32(buf: Buffer, offset = 0) {
  const adv = 4;
  return { elem: buf.readInt32BE(offset), adv };
}

/**
 * Deserialize a field element from a buffer, starting at the given offset.
 * The function reads 32 bytes from the buffer and converts it into a field element using Fr.fromBuffer.
 * It returns an object containing the deserialized field element and the number of bytes read (adv).
 *
 * @param buf - The buffer containing the serialized field element.
 * @param offset - The position in the buffer where the field element starts. Default is 0.
 * @returns An object with 'elem' as the deserialized field element and 'adv' as the number of bytes read.
 */
export function deserializeField(buf: Buffer, offset = 0) {
  const adv = 32;
  return { elem: Fr.fromBuffer(buf.subarray(offset, offset + adv)), adv };
}

/**
 * Serialize an array of Buffer instances into a single Buffer by concatenating the array length as a 4-byte unsigned integer
 * and then the individual Buffer elements. The function is useful for storing or transmitting an array of binary data chunks
 * (e.g., file parts) in a compact format.
 *
 * @param arr - An array of Buffer instances to be serialized into a single vector-like Buffer.
 * @returns A Buffer containing the serialized array length followed by the concatenated elements of the input Buffer array.
 */
export function serializeBufferArrayToVector(arr: Buffer[]) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(arr.length, 0);
  return Buffer.concat([lengthBuf, ...arr]);
}

/**
 * Deserialize an array of fixed length elements from a given buffer using a custom deserializer function.
 * The deserializer function should take the buffer and an offset as arguments, and return an object containing
 * the deserialized element and the number of bytes used to deserialize it (adv).
 *
 * @param deserialize - A custom deserializer function to extract individual elements from the buffer.
 * @param vector - The input buffer containing the serialized array.
 * @param offset - An optional starting position in the buffer for deserializing the array.
 * @returns An object containing the deserialized array and the total number of bytes used during deserialization (adv).
 */
export function deserializeArrayFromVector<T>(
  deserialize: (
    buf: Buffer,
    offset: number,
  ) => {
    /**
     * The element.
     */
    elem: T;
    /**
     * The advancement offset.
     */
    adv: number;
  },
  vector: Buffer,
  offset = 0,
) {
  let pos = offset;
  const size = vector.readUInt32BE(pos);
  pos += 4;
  const arr = new Array<T>(size);
  for (let i = 0; i < size; ++i) {
    const { elem, adv } = deserialize(vector, pos);
    pos += adv;
    arr[i] = elem;
  }
  return { elem: arr, adv: pos - offset };
}

/**
 * Parse a buffer as a big integer.
 */
export function toBigInt(buf: Buffer): bigint {
  const hex = buf.toString('hex');
  if (hex.length === 0) {
    return BigInt(0);
  }
  return BigInt(`0x${hex}`);
}
