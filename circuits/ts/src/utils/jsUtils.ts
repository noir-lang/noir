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
 * Assert a member is a certain length.
 * @param obj - An object.
 * @param member - A member string.
 * @param length - The length.
 */
export function assertLength<
  F extends string,
  T extends { [f in F]: { length: number } }
>(obj: T, member: F, length: number) {
  if (obj[member].length !== length) {
    throw new Error(
      `Expected ${member} to have length ${length}! Was: ${obj[member].length}`
    );
  }
}

/**
 * Asserts a collection has a certain length.
 * @param arr Collection to check.
 * @param expected Expected length.
 * @param label Optional name for the collection to use in the error message.
 */
export function checkLength(
  arr: { length: number },
  expected: number,
  label: string = "array"
) {
  if (arr.length !== expected) {
    throw new Error(
      `Invalid length for ${label} (got ${arr.length}, expected ${expected})`
    );
  }
}

/**
 * Strips methods of a type.
 */
export type FieldsOf<T> = {
  // eslint-disable-next-line @typescript-eslint/ban-types
  [P in keyof T as T[P] extends Function ? never : P]: T[P];
};
