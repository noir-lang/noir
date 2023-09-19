import { Buffer } from 'buffer';
import cloneDeepWith from 'lodash.clonedeepwith';

import { ClassConverter } from './class_converter.js';

/**
 * Check prototype chain to determine if an object is 'plain' (not a class instance).
 * @param obj - The object to check.
 * @returns True if the object is 'plain'.
 */
function isPlainObject(obj: any) {
  if (obj === null) {
    return false;
  }

  let proto = obj;
  let counter = 0;
  const MAX_PROTOTYPE_CHAIN_LENGTH = 1000; // Adjust as needed
  while (Object.getPrototypeOf(proto) !== null) {
    if (counter >= MAX_PROTOTYPE_CHAIN_LENGTH) {
      // This is a failsafe in case circular prototype chain has been created. It should not be hit
      return false;
    }
    proto = Object.getPrototypeOf(proto);
    counter++;
  }

  return Object.getPrototypeOf(obj) === proto;
}

/**
 * Recursively looks through an object for bigints and converts them to object format.
 * @param obj - The object to convert.
 * @returns The converted object with stringified bigints.
 */
export const convertBigintsInObj = (obj: any) => {
  return cloneDeepWith(obj, (value: any) => {
    if (typeof value === 'bigint') {
      return { type: 'bigint', data: value.toString() };
    }
  });
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
  if (typeof obj.type === 'string') {
    if (cc.isRegisteredClassName(obj.type)) {
      return cc.toClassObj(obj);
    } else {
      throw new Error(`Object ${obj.type} not registered for serialisation FROM JSON`);
    }
  }

  // Is this an array?
  if (Array.isArray(obj)) {
    return obj.map((x: any) => convertFromJsonObj(cc, x));
  }

  // Is this a dictionary?
  if (typeof obj === 'object') {
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
  // Bigint is a primitive type that needs special handling since it's not serialisable
  if (typeof obj === 'bigint') {
    return {
      type: 'bigint',
      data: obj.toString(),
    };
  }

  if (!obj) {
    return obj; // Primitive type
  }

  // Is this a Node buffer?
  if (obj instanceof Buffer) {
    return { type: 'Buffer', data: obj.toString('base64') };
  }

  // Is this a convertible type?
  if (cc.isRegisteredClass(obj.constructor)) {
    return cc.toJsonObj(obj);
  }

  // Is this an array?
  if (Array.isArray(obj)) {
    return obj.map((x: any) => convertToJsonObj(cc, x));
  }

  if (typeof obj === 'object') {
    // Is this a dictionary?
    if (isPlainObject(obj)) {
      const newObj: any = {};
      for (const key of Object.keys(obj)) {
        newObj[key] = convertToJsonObj(cc, obj[key]);
      }
      return newObj;
    } else {
      // Throw if this is a non-primitive class that was not registered
      throw new Error(`Object ${obj.constructor.name} not registered for serialisation TO JSON`);
    }
  }

  // Leave alone, assume JSON primitive
  return obj;
}
