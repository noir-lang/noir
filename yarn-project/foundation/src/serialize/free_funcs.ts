import { Fr } from '../fields/fields.js';
import { type Tuple } from './types.js';

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
 * @param n - The input number to be converted to a big-endian unsigned 16-bit integer Buffer.
 * @param bufferSize - Optional, the size of the output Buffer (default is 2).
 * @returns A Buffer containing the big-endian unsigned 16-bit integer representation of the input number.
 */
export function numToUInt16BE(n: number, bufferSize = 2) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt16BE(n, bufferSize - 2);
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
 * Adds a 4-byte byte-length prefix to a buffer.
 * @param buf - The input Buffer to be prefixed
 * @returns A Buffer with 4-byte byte-length prefix.
 */
export function prefixBufferWithLength(buf: Buffer) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(buf.length, 0);
  return Buffer.concat([lengthBuf, buf]);
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

/**
 * Stores full 256 bits of information in 2 fields.
 * @param buf - 32 bytes of data
 * @returns 2 field elements
 */
export function to2Fields(buf: Buffer): [Fr, Fr] {
  if (buf.length !== 32) {
    throw new Error('Buffer must be 32 bytes');
  }

  // Split the hash into two fields, a high and a low
  const buf1 = Buffer.concat([Buffer.alloc(16), buf.subarray(0, 16)]);
  const buf2 = Buffer.concat([Buffer.alloc(16), buf.subarray(16, 32)]);

  return [Fr.fromBuffer(buf1), Fr.fromBuffer(buf2)];
}

/**
 * Reconstructs the original 32 bytes of data from 2 field elements.
 * @param field1 - First field element
 * @param field2 - Second field element
 * @returns 32 bytes of data as a Buffer
 */
export function from2Fields(field1: Fr, field2: Fr): Buffer {
  // Convert the field elements back to buffers
  const buf1 = field1.toBuffer();
  const buf2 = field2.toBuffer();

  // Remove the padding (first 16 bytes) from each buffer
  const originalPart1 = buf1.subarray(Fr.SIZE_IN_BYTES / 2, Fr.SIZE_IN_BYTES);
  const originalPart2 = buf2.subarray(Fr.SIZE_IN_BYTES / 2, Fr.SIZE_IN_BYTES);

  // Concatenate the two parts to form the original buffer
  return Buffer.concat([originalPart1, originalPart2]);
}

/**
 * Truncates SHA hashes to match Noir's truncated version
 * @param buf - 32 bytes of data
 * @returns 31 bytes of data padded to 32
 */
export function truncateAndPad(buf: Buffer): Buffer {
  // Note that we always truncate here, to match solidity's sha256ToField()
  if (buf.length !== 32) {
    throw new Error('Buffer to truncate must be 32 bytes');
  }
  return Buffer.concat([Buffer.alloc(1), buf.subarray(0, 31)]);
}

/**
 * Stores 248 bits of information in 1 field.
 * @param buf - 32 or 31 bytes of data
 * @returns 1 field element
 */
export function toTruncField(buf: Buffer): Fr {
  if (buf.length !== 32 && buf.length !== 31) {
    throw new Error('Buffer must be 31 or 32 bytes');
  }
  if ((buf.length == 32 && buf[0] == 0) || buf.length == 31) {
    return Fr.fromBuffer(buf);
  } else {
    // Note: safer to NOT truncate here, all inputs are expected to be truncated
    // from Noir or L1 Contracts or Class.hash() methods
    throw new Error(`Number ${toBigInt(buf)} does not fit in 31 byte truncated buffer`);
  }
}

/**
 * Reconstructs the original 31 bytes of data from 1 truncated field element.
 * @param field - field element
 * @returns 31 bytes of data as a Buffer
 */
export function fromTruncField(field: Fr): Buffer {
  const buffer = field.toBuffer();
  if (buffer[0] != 0) {
    throw new Error(`Number ${field} does not fit in 31 byte truncated buffer`);
  }
  return buffer;
}

export function fromFieldsTuple(fields: Tuple<Fr, 2>): Buffer {
  return from2Fields(fields[0], fields[1]);
}

export function toHumanReadable(buf: Buffer, maxLen?: number): string {
  const result = buf.every(byte => byte >= 32 && byte <= 126) ? buf.toString('ascii') : `0x${buf.toString('hex')}`;
  if (maxLen && result.length > maxLen) {
    return result.slice(0, maxLen) + '...';
  }
  return result;
}
