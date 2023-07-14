import { Buffer } from 'buffer';

import { ClassConverter } from './class_converter.js';

/**
 * Recursively looks through an object for bigints and converts them to object format.
 * @param obj - The object to convert.
 * @returns The converted object with stringified bigints.
 */
export const convertBigintsInObj = (obj: any) => {
  for (const i in obj) {
    if (typeof obj[i] === 'bigint') {
      obj[i] = {
        type: 'bigint',
        data: obj[i].toString(),
      };
    } else if (typeof obj[i] === 'object') {
      convertBigintsInObj(obj[i]);
    }
  }
  return obj;
};

/**
 * JSON.stringify helper that handles bigints.
 * @param obj - The object to be stringified.
 * @returns The resulting string.
 */
export function JsonStringify(obj: object, prettify?: boolean): string {
  return JSON.stringify(
    obj,
    (key, value) =>
      typeof value === 'bigint'
        ? JSON.stringify({
            type: 'bigint',
            data: value.toString(),
          })
        : value,
    prettify ? 2 : 0,
  );
}

/**
 * Convert a JSON-friendly object, which may encode a class object.
 * @param cc - The class converter.
 * @param obj - The encoded object.
 * @returns The decoded object.
 */
export function convertFromJsonObj(cc: ClassConverter, obj: any): any {
  if (obj === null) {
    return undefined; // `null` doesn't work with default args.
  }

  if (!obj) {
    return obj; // Primitive type
  }
  // Is this a serialized Node buffer?
  if (obj.type === 'Buffer' && typeof obj.data === 'string') {
    return Buffer.from(obj.data, 'base64');
  }

  if (obj.type === 'bigint' && typeof obj.data === 'string') {
    return BigInt(obj.data);
  }

  // Is this a convertible type?
  if (typeof obj.type === 'string' && cc.isRegisteredClassName(obj.type)) {
    return cc.toClassObj(obj);
  }

  // Is this an array?
  if (Array.isArray(obj)) {
    return obj.map((x: any) => convertFromJsonObj(cc, x));
  }
  // Is this a dictionary?
  if (obj.constructor === Object) {
    const newObj: any = {};
    for (const key of Object.keys(obj)) {
      newObj[key] = convertFromJsonObj(cc, obj[key]);
    }
    return newObj;
  }

  // Leave alone, assume JSON primitive
  return obj;
}

/**
 * Convert objects or classes to a JSON-friendly object.
 * @param cc - The class converter.
 * @param obj - The object.
 * @returns The encoded object.
 */
export function convertToJsonObj(cc: ClassConverter, obj: any): any {
  if (!obj) {
    return obj; // Primitive type
  }
  // Is this a Node buffer?
  if (obj instanceof Buffer) {
    return { type: 'Buffer', data: obj.toString('base64') };
  }

  if (typeof obj === 'bigint') {
    return {
      type: 'bigint',
      data: obj.toString(),
    };
  }

  // Is this a convertible type?
  if (cc.isRegisteredClass(obj.constructor)) {
    return cc.toJsonObj(obj);
  }
  // Is this an array?
  if (Array.isArray(obj)) {
    return obj.map((x: any) => convertToJsonObj(cc, x));
  }
  // Is this a dictionary?
  if (obj.constructor === Object) {
    const newObj: any = {};
    for (const key of Object.keys(obj)) {
      newObj[key] = convertToJsonObj(cc, obj[key]);
    }
    return newObj;
  }

  // Leave alone, assume JSON primitive
  return obj;
}
