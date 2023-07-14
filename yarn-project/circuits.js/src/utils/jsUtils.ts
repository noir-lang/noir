import { Tuple } from '@aztec/foundation/serialize';

export type { FieldsOf } from '@aztec/foundation/types';

/**
 * Create an array over an integer range.
 * @param n - The number of integers.
 * @param offset - The starting number.
 * @returns The array of numbers.
 */
export function range(n: number, offset = 0) {
  const ret: number[] = [];
  for (let i = 0; i < n; i++) {
    ret.push(offset + i);
  }
  return ret;
}

/**
 * Create an array over an integer range, filled with a function 'fn'.
 * This is used over e.g. lodash because it resolved to a tuple type, needed for our fixed array type safety.
 * @param n - The number of integers.
 * @param fn - The generator function.
 * @returns The array of numbers.
 */
export function makeTuple<T, N extends number>(length: N, fn: (i: number) => T, offset = 0) {
  return Array.from({ length }, (v: any, i: number) => fn(i + offset)) as Tuple<T, N>;
}

/**
 * Create an array over an integer range, filled with a function 'fn'. However, the latter half of the array are set to zeros.
 * see `makeTuple` above.
 * @param n - The number of integers.
 * @param fn - The generator function.
 * @returns The array of numbers.
 */
export function makeHalfFullTuple<T, N extends number>(length: N, fn: (i: number) => T, offset = 0) {
  return Array.from({ length }, (v: any, i: number) => (i < length / 2 ? fn(i + offset) : fn(0))) as Tuple<T, N>;
}

/**
 * Assert a member of an object is a certain length.
 * @param obj - An object.
 * @param member - A member string.
 * @param length - The length.
 */
export function assertMemberLength<
  F extends string,
  T extends {
    [f in F]: {
      /**
       * A property which the tested member of the object T has to have.
       */
      length: number;
    };
  },
>(obj: T, member: F, length: number) {
  if (obj[member].length !== length) {
    throw new Error(`Expected ${member} to have length ${length} but was ${obj[member].length}`);
  }
}

/**
 * Assert all subarrays in a member of an object are a certain length.
 * @param obj - An object.
 * @param member - A member string.
 * @param length - The expected length for each subarray.
 */
export function assertItemsLength<
  F extends string,
  T extends {
    [f in F]: {
      /**
       * A property which the tested member of the object T has to have.
       */
      length: number;
    }[];
  },
>(obj: T, member: F, length: number) {
  const arrs = obj[member];
  for (let i = 0; i < arrs.length; i++) {
    if (arrs[i].length !== length) {
      throw new Error(`Expected ${member}[${i}] to have length ${length} but was ${arrs[i].length}`);
    }
  }
}
