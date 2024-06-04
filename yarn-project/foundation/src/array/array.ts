import { type Tuple } from '../serialize/index.js';

export type { FieldsOf } from '../types/index.js';

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
  return Array.from({ length }, (_: any, i: number) => fn(i + offset)) as Tuple<T, N>;
}

/**
 * Create an array over an integer range, filled with a function 'fn'. However, the latter half of the array are set to zeros.
 * see `makeTuple` above.
 * @param n - The number of integers.
 * @param fn - The generator function.
 * @returns The array of numbers.
 */
export function makeHalfFullTuple<T, N extends number>(
  length: N,
  fn: (i: number) => T,
  offset = 0,
  makeEmpty: () => T,
) {
  return Array.from({ length }, (_: any, i: number) => (i < length / 2 ? fn(i + offset) : makeEmpty())) as Tuple<T, N>;
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
  const arrays = obj[member];
  for (let i = 0; i < arrays.length; i++) {
    if (arrays[i].length !== length) {
      throw new Error(`Expected ${member}[${i}] to have length ${length} but was ${arrays[i].length}`);
    }
  }
}

/**
 * Checks that the permutation is valid. Throws an error if it is not.
 * @param original - The original array.
 * @param permutation - The array which is allegedly a permutation of the original.
 * @param indexes - The indices of the original array which the permutation should map to.
 * @param isEqual - A function to compare the elements of the original and permutation arrays.
 */
export function assertPermutation<T>(
  original: T[],
  permutation: T[],
  indexes: number[],
  isEqual: (a: T, b: T) => boolean,
): void {
  if (original.length !== permutation.length || original.length !== indexes.length) {
    throw new Error(`Invalid lengths: ${original.length}, ${permutation.length}, ${indexes.length}`);
  }

  const seenValue = new Set<number>();
  for (let i = 0; i < indexes.length; i++) {
    const index = indexes[i];
    const permutedValue = permutation[i];
    const originalValueAtIndex = original[index];

    if (!isEqual(permutedValue, originalValueAtIndex)) {
      throw new Error(`Invalid permutation at index ${index}: ${permutedValue} !== ${originalValueAtIndex}`);
    }
    if (seenValue.has(index)) {
      throw new Error(`Duplicate index in permutation: ${index}`);
    }
    seenValue.add(index);
  }
}

export function assertRightPadded<T>(arr: T[], isEmpty: (item: T) => boolean) {
  let seenEmpty = false;
  for (let i = 0; i < arr.length; i++) {
    if (isEmpty(arr[i])) {
      seenEmpty = true;
    } else if (seenEmpty) {
      throw new Error(`Non-empty element at index [${i}] after empty element`);
    }
  }
}
