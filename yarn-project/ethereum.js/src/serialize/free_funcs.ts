import { toBigIntBE, toBufferBE } from '../bigint_buffer/index.js';

// For serializing bool.
/**
 *
 */
export function boolToByte(b: boolean) {
  const buf = Buffer.alloc(1);
  buf.writeUInt8(b ? 1 : 0);
  return buf;
}

// For serializing numbers to 32 bit little-endian form.
/**
 *
 */
export function numToUInt32LE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32LE(n, bufferSize - 4);
  return buf;
}

// For serializing numbers to 32 bit big-endian form.
/**
 *
 */
export function numToUInt32BE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32BE(n, bufferSize - 4);
  return buf;
}

// For serializing signed numbers to 32 bit big-endian form.
/**
 *
 */
export function numToInt32BE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeInt32BE(n, bufferSize - 4);
  return buf;
}

// For serializing numbers to 32 bit big-endian form.
/**
 *
 */
export function numToUInt8(n: number) {
  const bufferSize = 1;
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt8(n, 0);
  return buf;
}

// For serializing a buffer as a vector.
/**
 *
 */
export function serializeBufferToVector(buf: Buffer) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(buf.length, 0);
  return Buffer.concat([lengthBuf, buf]);
}

/**
 * Serialize a BigInt value into a buffer with specified width (number of bytes).
 * The output buffer represents the big-endian encoding of the input BigInt value.
 * If the width is not provided, it defaults to 32 bytes.
 *
 * @param n - The BigInt value to be serialized.
 * @param width - The number of bytes for the output buffer (optional, default: 32).
 * @returns A Buffer containing the serialized big-endian representation of the BigInt value.
 */
export function serializeBigInt(n: bigint, width = 32) {
  return toBufferBE(n, width);
}

/**
 * Deserialize a bigint from a buffer with a specified offset and width.
 * The function extracts a slice of the buffer based on the given offset and width,
 * and converts that slice into a bigint value.
 *
 * @param buf - The input buffer containing the serialized bigint data.
 * @param offset - The starting index within the buffer to deserialize from. Default is 0.
 * @param width - The number of bytes to use for deserialization. Default is 32.
 * @returns An object containing the deserialized bigint ('elem') and the advancement ('adv') in the buffer.
 */
export function deserializeBigInt(buf: Buffer, offset = 0, width = 32) {
  return { elem: toBigIntBE(buf.slice(offset, offset + width)), adv: width };
}

/**
 * Serialize a given JavaScript Date object into an 8-byte big-endian BigInt buffer.
 * The function first converts the date to its corresponding UNIX timestamp (in milliseconds),
 * then creates a BigInt from the timestamp and serializes it to an 8-byte buffer using
 * big-endian format. This buffer can be useful for interoperability with other systems and
 * data storage.
 *
 * @param date - The JavaScript Date object to be serialized.
 * @returns A Buffer containing the serialized date as an 8-byte big-endian BigInt.
 */
export function serializeDate(date: Date) {
  return serializeBigInt(BigInt(date.getTime()), 8);
}

/**
 * Deserialize a Buffer from a vector, given its starting offset.
 * This function reads the length of the buffer from the vector and extracts the corresponding bytes.
 * It returns an object containing the deserialized Buffer element and the total number of bytes advanced (including the length).
 *
 * @param vector - The input vector Buffer from which the buffer will be deserialized.
 * @param offset - The starting offset in the input vector where the buffer begins. Default is 0.
 * @returns An object with the deserialized Buffer element as 'elem' and the total number of bytes advanced as 'adv'.
 */
export function deserializeBufferFromVector(vector: Buffer, offset = 0) {
  const length = vector.readUInt32BE(offset);
  const adv = 4 + length;
  return { elem: vector.slice(offset + 4, offset + adv), adv };
}

/**
 * Deserialize a boolean value from a buffer at the specified offset.
 * The function reads one byte from the buffer and returns an object with the deserialized boolean value and the number of bytes advanced (adv).
 *
 * @param buf - The buffer containing the serialized boolean value.
 * @param offset - The starting position in the buffer to read the boolean value. Default is 0.
 * @returns An object with the deserialized boolean value (elem) and the number of bytes advanced (adv).
 */
export function deserializeBool(buf: Buffer, offset = 0) {
  const adv = 1;
  return { elem: buf.readUInt8(offset), adv };
}

/**
 * Deserialize a 32-bit unsigned integer from a Buffer.
 * Extracts a 32-bit unsigned integer from the provided Buffer at the specified offset and
 * returns the deserialized value along with the number of bytes advanced in the buffer.
 *
 * @param buf - The source Buffer to deserialize the unsigned integer from.
 * @param offset - The starting position within the Buffer to read the unsigned integer.
 * @returns An object containing the deserialized 32-bit unsigned integer (elem) and number of bytes advanced (adv).
 */
export function deserializeUInt32(buf: Buffer, offset = 0) {
  const adv = 4;
  return { elem: buf.readUInt32BE(offset), adv };
}

/**
 * Deserialize a signed 32-bit integer from the given buffer at the specified offset.
 * Returns the deserialized integer value along with the number of bytes advanced (4) as an object.
 *
 * @param buf - The input buffer containing the binary data to deserialize.
 * @param offset - The optional starting position (index) in the buffer for deserialization. Default is 0.
 * @returns An object containing the deserialized signed 32-bit integer ('elem') and the number of bytes advanced ('adv', always 4).
 */
export function deserializeInt32(buf: Buffer, offset = 0) {
  const adv = 4;
  return { elem: buf.readInt32BE(offset), adv };
}

/**
 * Deserialize a field from a given buffer starting at the specified offset.
 * This function reads a fixed size (32 bytes) slice of the buffer and returns an object containing
 * the extracted field as a Buffer and the number of bytes advanced in the input buffer.
 *
 * @param buf - The input buffer containing the serialized data.
 * @param offset - The starting position in the buffer to begin deserialization (default is 0).
 * @returns An object containing the extracted field as a Buffer and the number of bytes advanced in the input buffer.
 */
export function deserializeField(buf: Buffer, offset = 0) {
  const adv = 32;
  return { elem: buf.slice(offset, offset + adv), adv };
}

// For serializing an array of fixed length elements.
/**
 *
 */
export function serializeBufferArrayToVector(arr: Buffer[]) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(arr.length, 0);
  return Buffer.concat([lengthBuf, ...arr]);
}

/**
 * Deserialize an array of fixed length elements from a buffer, given a deserialization function.
 * Reads the size of the array from the buffer at the provided offset, then iterates through the
 * elements and applies the provided deserialization function on each element. Returns an array
 * of the deserialized elements and the total bytes consumed in the process.
 *
 * @typeparam T - The type of the deserialized elements.
 * @param deserialize - The deserialization function to be applied on each element of the array.
 * @param vector - The source buffer containing the serialized data.
 * @param offset - The starting position in the buffer to begin deserialization (optional, default is 0).
 * @returns An object containing the deserialized array of elements (elem) and the total bytes consumed in the process (adv).
 */
export function deserializeArrayFromVector<T>(
  deserialize: (
    buf: Buffer,
    offset: number,
  ) => {
    /**
     * The deserialized element from the buffer.
     */
    elem: T;
    /**
     * The number of bytes advanced in the buffer during deserialization.
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
