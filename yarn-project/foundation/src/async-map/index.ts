/**
 * Maps an array of elements by applying an asynchronous function to each element in sequence,
 * and returns a new array with the results. The function receives each element of the array
 * and its index as arguments, and should return a Promise that resolves to the desired value.
 *
 * @typeParam T - The original array element type.
 * @typeParam U - The resulting array element type.
 * @param arr - The array to map.
 * @param fn - The async function to apply on each element of the array.
 * @returns A Promise that resolves to a new array containing the mapped values.
 */
export async function asyncMap<T, U>(arr: T[], fn: (e: T, i: number) => Promise<U>): Promise<U[]> {
  const results: U[] = [];
  for (let i = 0; i < arr.length; ++i) {
    results.push(await fn(arr[i], i));
  }
  return results;
}
