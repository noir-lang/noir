import { RawBuffer } from '../types/raw_buffer.js';

// For serializing bool.
export function boolToBuffer(b: boolean) {
  const buf = new Uint8Array(1);
  buf[0] = b ? 1 : 0;
  return buf;
}

// For serializing numbers to 32 bit little-endian form.
export function numToUInt32LE(n: number, bufferSize = 4) {
  const buf = new Uint8Array(bufferSize);
  new DataView(buf.buffer).setUint32(buf.byteLength - 4, n, true);
  return buf;
}

// For serializing numbers to 32 bit big-endian form.
export function numToUInt32BE(n: number, bufferSize = 4) {
  const buf = new Uint8Array(bufferSize);
  new DataView(buf.buffer).setUint32(buf.byteLength - 4, n, false);
  return buf;
}

// For serializing signed numbers to 32 bit big-endian form.
export function numToInt32BE(n: number, bufferSize = 4) {
  const buf = new Uint8Array(bufferSize);
  new DataView(buf.buffer).setInt32(buf.byteLength - 4, n, false);
  return buf;
}

// For serializing numbers to 8 bit form.
export function numToUInt8(n: number) {
  const buf = new Uint8Array(1);
  buf[0] = n;
  return buf;
}

export function concatenateUint8Arrays(arrayOfUint8Arrays: Uint8Array[]) {
  const totalLength = arrayOfUint8Arrays.reduce((prev, curr) => prev + curr.length, 0);
  const result = new Uint8Array(totalLength);
  let length = 0;
  for (const array of arrayOfUint8Arrays) {
    result.set(array, length);
    length += array.length;
  }
  return result;
}

export function uint8ArrayToHexString(uint8Array: Uint8Array) {
  return uint8Array.reduce((accumulator, byte) => accumulator + byte.toString(16).padStart(2, '0'), '');
}

// For serializing a buffer as a vector.
export function serializeBufferToVector(buf: Uint8Array) {
  return concatenateUint8Arrays([numToInt32BE(buf.length), buf]);
}

export function serializeBigInt(n: bigint, width = 32) {
  const buf = new Uint8Array(width);
  for (let i = 0; i < width; i++) {
    buf[width - i - 1] = Number((n >> BigInt(i * 8)) & 0xffn);
  }
  return buf;
}

export function deserializeBigInt(buf: Uint8Array, offset = 0, width = 32) {
  let result = 0n;
  for (let i = 0; i < width; i++) {
    result = (result << BigInt(8)) | BigInt(buf[offset + i]);
  }
  return { elem: result, adv: width };
}

export function serializeDate(date: Date) {
  return serializeBigInt(BigInt(date.getTime()), 8);
}

export function deserializeBufferFromVector(vector: Uint8Array, offset = 0) {
  const length = new DataView(vector.buffer, vector.byteOffset + offset, 4).getUint32(0, false);
  const adv = 4 + length;
  const elem = vector.slice(offset + 4, offset + adv);
  return { elem, adv };
}

export function deserializeBool(buf: Uint8Array, offset = 0) {
  const adv = 1;
  const elem = buf[offset] !== 0;
  return { elem, adv };
}

export function deserializeUInt32(buf: Uint8Array, offset = 0) {
  const adv = 4;
  const elem = new DataView(buf.buffer, buf.byteOffset + offset, adv).getUint32(0, false);
  return { elem, adv };
}

export function deserializeInt32(buf: Uint8Array, offset = 0) {
  const adv = 4;
  const elem = new DataView(buf.buffer, buf.byteOffset + offset, adv).getInt32(0, false);
  return { elem, adv };
}

export function deserializeField(buf: Uint8Array, offset = 0) {
  const adv = 32;
  const elem = buf.slice(offset, offset + adv);
  return { elem, adv };
}

// For serializing an array of fixed length elements.
export function serializeBufferArrayToVector(arr: Uint8Array[]) {
  return concatenateUint8Arrays([numToUInt32BE(arr.length), ...arr.flat()]);
}

export function deserializeArrayFromVector<T>(
  deserialize: (buf: Uint8Array, offset: number) => { elem: T; adv: number },
  vector: Uint8Array,
  offset = 0,
) {
  let pos = offset;
  const size = new DataView(vector.buffer, vector.byteOffset + pos, 4).getUint32(0, false);
  pos += 4;
  const arr = new Array<T>(size);
  for (let i = 0; i < size; ++i) {
    const { elem, adv } = deserialize(vector, pos);
    pos += adv;
    arr[i] = elem;
  }
  return { elem: arr, adv: pos - offset };
}

/** A type that can be written to a buffer. */
export type Bufferable = boolean | Uint8Array | number | string | { toBuffer: () => Uint8Array } | Bufferable[];

/**
 * Serializes a list of objects contiguously for calling into wasm.
 * @param objs - Objects to serialize.
 * @returns A buffer list with the concatenation of all fields.
 */
export function serializeBufferable(obj: Bufferable): Uint8Array {
  if (Array.isArray(obj)) {
    return serializeBufferArrayToVector(obj.map(serializeBufferable));
  } else if (obj instanceof RawBuffer) {
    return obj;
  } else if (obj instanceof Uint8Array) {
    return serializeBufferToVector(obj);
  } else if (typeof obj === 'boolean') {
    return boolToBuffer(obj);
  } else if (typeof obj === 'number') {
    return numToUInt32BE(obj);
  } else if (typeof obj === 'bigint') {
    return serializeBigInt(obj);
  } else if (typeof obj === 'string') {
    return serializeBufferToVector(new TextEncoder().encode(obj));
  } else {
    return obj.toBuffer();
  }
}
