import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/fields.js';
import { numToUInt32BE } from './free_funcs.js';

/**
 * For serializing an array of fixed length buffers.
 * @param arr - Array of bufferable.
 * @param prefixLength - The length of the prefix (denominated in bytes).
 * @returns The serialized buffers.
 */
export function serializeArrayOfBufferableToVector(objs: Bufferable[], prefixLength = 4): Buffer {
  const arr = serializeToBufferArray(objs);
  let lengthBuf: Buffer;
  if (prefixLength === 1) {
    lengthBuf = Buffer.alloc(1);
    lengthBuf.writeUInt8(arr.length, 0);
  } else if (prefixLength === 4) {
    lengthBuf = Buffer.alloc(4);
    lengthBuf.writeUInt32BE(arr.length, 0);
  } else {
    throw new Error(`Unsupported prefix length. Got ${prefixLength}, expected 1 or 4`);
  }
  return Buffer.concat([lengthBuf, ...arr]);
}

/**
 * Helper function for deserializeArrayFromVector.
 */
type DeserializeFn<T> = (
  buf: Buffer,
  offset: number,
) => {
  /**
   * The deserialized type.
   */
  elem: T;
  /**
   * How many bytes to advance by.
   */
  adv: number;
};

/**
 * Deserializes an array from a vector on an element-by-element basis.
 * @param deserialize - A function used to deserialize each element of the vector.
 * @param vector - The vector to deserialize.
 * @param offset - The position in the vector to start deserializing from.
 * @returns Deserialized array and how many bytes we advanced by.
 */
export function deserializeArrayFromVector<T>(
  deserialize: DeserializeFn<T>,
  vector: Buffer,
  offset = 0,
): {
  /**
   * The deserialized array.
   */
  elem: T[];
  /**
   * How many bytes we advanced by.
   */
  adv: number;
} {
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
 * Cast a uint8 array to a number.
 * @param array - The uint8 array.
 * @returns The number.
 */
export function uint8ArrayToNum(array: Uint8Array): number {
  const buf = Buffer.from(array);
  return buf.readUint32LE();
}

/**
 * Serializes a boolean to a buffer.
 * @param value - Value to serialize.
 * @returns The serialized boolean.
 */
export function boolToBuffer(value: boolean, bufferSize = 1): Buffer {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt8(value ? 1 : 0, bufferSize - 1);
  return buf;
}

/**
 * Deserialize the 256-bit number at address `offset`.
 * @param buf - The buffer.
 * @param offset - The address.
 * @returns The deserialized 256-bit field.
 */
export function deserializeField(buf: Buffer, offset = 0) {
  const adv = 32;
  return { elem: buf.slice(offset, offset + adv), adv };
}

/** A type that can be written to a buffer. */
export type Bufferable =
  | boolean
  | Buffer
  | number
  | string
  | {
      /**
       * Serialize to a buffer.
       */
      toBuffer: () => Buffer;
    }
  | Bufferable[];

/** A type that can be converted to a Field or a Field array. */
export type Fieldeable =
  | Fr
  | boolean
  | number
  | bigint
  | {
      /** Serialize to a field. */
      toField: () => Fr;
    }
  | {
      /** Serialize to an array of fields. */
      toFields: () => Fr[];
    }
  | Fieldeable[];

/**
 * Serializes a list of objects contiguously.
 * @param objs - Objects to serialize.
 * @returns A buffer list with the concatenation of all fields.
 */
export function serializeToBufferArray(...objs: Bufferable[]): Buffer[] {
  let ret: Buffer[] = [];
  for (const obj of objs) {
    if (Array.isArray(obj)) {
      ret = [...ret, ...serializeToBufferArray(...obj)];
    } else if (Buffer.isBuffer(obj)) {
      ret.push(obj);
    } else if (typeof obj === 'boolean') {
      ret.push(boolToBuffer(obj));
    } else if (typeof obj === 'number') {
      // Note: barretenberg assumes everything is big-endian
      ret.push(numToUInt32BE(obj)); // TODO: Are we always passing numbers as UInt32?
    } else if (typeof obj === 'string') {
      ret.push(numToUInt32BE(obj.length));
      ret.push(Buffer.from(obj));
    } else {
      ret.push(obj.toBuffer());
    }
  }
  return ret;
}

/**
 * Serializes a list of objects contiguously.
 * @param objs - Objects to serialize.
 * @returns An array of fields with the concatenation of all fields.
 */
export function serializeToFields(...objs: Fieldeable[]): Fr[] {
  let ret: Fr[] = [];
  for (const obj of objs) {
    if (Array.isArray(obj)) {
      ret = [...ret, ...serializeToFields(...obj)];
    } else if (obj instanceof Fr) {
      ret.push(obj);
    } else if (typeof obj === 'boolean' || typeof obj === 'number' || typeof obj === 'bigint') {
      ret.push(new Fr(obj));
    } else if ('toFields' in obj) {
      ret = [...ret, ...obj.toFields()];
    } else {
      ret.push(obj.toField());
    }
  }
  return ret;
}

/**
 * Serializes a list of objects contiguously.
 * @param objs - Objects to serialize.
 * @returns A single buffer with the concatenation of all fields.
 */
export function serializeToBuffer(...objs: Bufferable[]): Buffer {
  return Buffer.concat(serializeToBufferArray(...objs));
}

/**
 * Returns a user-friendly JSON representation of an object, showing buffers as hex strings.
 * @param obj - Object to json-stringify.
 * @returns A JSON string.
 */
export function toFriendlyJSON(obj: object): string {
  return JSON.stringify(
    obj,
    (key, value) => {
      if (value !== null && typeof value === 'object' && value.type === 'Buffer' && Array.isArray(value.data)) {
        return '0x' + Buffer.from(value.data).toString('hex');
      } else if (typeof value === 'bigint') {
        return value.toString();
      } else if (
        value &&
        (
          value as {
            /**
             * Signature of the target serialization function.
             */
            toFriendlyJSON: () => string;
          }
        ).toFriendlyJSON
      ) {
        return value.toFriendlyJSON();
      } else {
        return value;
      }
    },
    2,
  );
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
