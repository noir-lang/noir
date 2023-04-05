/**
 * For serializing an array of fixed length buffers.
 * TODO move to foundation pkg.
 * @param arr - Array of bufffers.
 * @returns The serialized buffers.
 */
export function serializeBufferArrayToVector(arr: Buffer[]) {
  const lengthBuf = Buffer.alloc(4);
  lengthBuf.writeUInt32BE(arr.length, 0);
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
 * For deserializing numbers to 32-bit little-endian form.
 * TODO move to foundation pkg.
 * @param n - The number.
 * @returns The endian-corrected number.
 */
export function deserializeArrayFromVector<T>(deserialize: DeserializeFn<T>, vector: Buffer, offset = 0) {
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
 * For serializing numbers to 32 bit little-endian form.
 * TODO move to foundation pkg.
 * @param n - The number.
 * @returns The endian-corrected number.
 */
export function numToUInt32LE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32LE(n, bufferSize - 4);
  return buf;
}

/**
 * For serializing numbers to 32 bit big-endian form.
 * TODO move to foundation pkg.
 * @param n - The number.
 * @returns The endian-corrected number.
 */
export function numToUInt32BE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32BE(n, bufferSize - 4);
  return buf;
}

/**
 * Cast a uint8 array to a number.
 * @param array - The uint8 array.
 * @returns The number.
 */
export function uint8ArrayToNum(array: Uint8Array) {
  const buf = Buffer.from(array);
  return buf.readUint32LE();
}

/**
 * For serializing booleans in structs for calling into wasm.
 * @param bool - Value to serialize.
 */
export function boolToBuffer(value: boolean) {
  return Buffer.from([value ? 1 : 0]);
}

/**
 * Deserialize the 256-bit number at address `offset`.
 * @param buf - The buffer.
 * @param offset - The address.
 * @returns The derserialized 256-bit field.
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
       * Serialize to a buffer of 32 bytes.
       */
      toBuffer32: () => Buffer;
    }
  | {
      /**
       * Serialize to a buffer.
       */
      toBuffer: () => Buffer;
    }
  | Bufferable[];

function isSerializableToBuffer32(obj: object): obj is { toBuffer32: () => Buffer } {
  return !!(obj as { toBuffer32: () => Buffer }).toBuffer32;
}

/**
 * Serializes a list of objects contiguously for calling into wasm.
 * @param objs - Objects to serialize.
 * @returns A buffer list with the concatenation of all fields.
 */
export function serializeToBufferArray(...objs: Bufferable[]): Buffer[] {
  let ret: Buffer[] = [];
  for (const obj of objs) {
    if (Array.isArray(obj)) {
      // Note: These must match the length of the C++ structs
      ret = [...ret, ...serializeToBufferArray(...obj)];
    } else if (Buffer.isBuffer(obj)) {
      ret.push(obj);
    } else if (typeof obj === 'boolean') {
      ret.push(boolToBuffer(obj));
    } else if (typeof obj === 'number') {
      // Note: barretenberg assumes everything is big-endian
      ret.push(numToUInt32BE(obj)); // TODO: Are we always passsing numbers as UInt32?
    } else if (typeof obj === 'string') {
      ret.push(numToUInt32BE(obj.length));
      ret.push(Buffer.from(obj));
    } else if (isSerializableToBuffer32(obj)) {
      ret.push(obj.toBuffer32());
    } else {
      ret.push(obj.toBuffer());
    }
  }
  return ret;
}

/**
 * Serializes a list of objects contiguously for calling into wasm.
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
      } else if ((value as { toFriendlyJSON: () => string }).toFriendlyJSON) {
        return value.toFriendlyJSON();
      } else {
        return value;
      }
    },
    2,
  );
}
