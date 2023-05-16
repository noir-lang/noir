/**
 * Pads an array to the target length by appending an element to its end. Throws if target length exceeds the input array length. Does not modify the input array.
 * @param arr - Array with elements to pad.
 * @param elem - Element to use for padding.
 * @param length - Target length.
 * @returns A new padded array.
 */
export function padArrayEnd<T>(arr: T[], elem: T, length: number): T[] {
  if (arr.length > length) throw new Error(`Array size exceeds target length`);
  return [...arr, ...Array(length - arr.length).fill(elem)];
}

/**
 * Pads an array to the target length by prepending elements at the beginning. Throws if target length exceeds the input array length. Does not modify the input array.
 * @param arr - Array with elements to pad.
 * @param elem - Element to use for padding.
 * @param length - Target length.
 * @returns A new padded array.
 */
export function padArrayStart<T>(arr: T[], elem: T, length: number): T[] {
  if (arr.length > length) throw new Error(`Array size exceeds target length`);
  return [...Array(length - arr.length).fill(elem), ...arr];
}

/**
 * Returns if an array is composed of empty items.
 * @param arr - Array to check.
 * @returns True if every item in the array isEmpty.
 */
export function isArrayEmpty<T>(arr: T[], isEmpty: (item: T) => boolean): boolean {
  for (const item of arr) {
    if (!isEmpty(item)) return false;
  }
  return true;
}

/**
 * Returns the number of non-empty items in an array.
 * @param arr - Array to check.
 * @returns Number of non-empty items in an array.
 */
export function arrayNonEmptyLength<T>(arr: T[], isEmpty: (item: T) => boolean): number {
  return arr.reduce((sum, item) => (isEmpty(item) ? sum : sum + 1), 0);
}
