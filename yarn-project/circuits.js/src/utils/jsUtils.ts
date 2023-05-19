import { Tuple } from '@aztec/foundation/serialize';

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
    throw new Error(`Expected ${member} to have length ${length}! Was: ${obj[member].length}`);
  }
}

/**
 * Strips methods of a type.
 */
export type FieldsOf<T> = {
  // eslint-disable-next-line @typescript-eslint/ban-types
  [P in keyof T as T[P] extends Function ? never : P]: T[P];
};
