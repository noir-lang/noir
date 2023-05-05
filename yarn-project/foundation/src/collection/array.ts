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
