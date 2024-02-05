/**
 * Represents a fixed-length array.
 */
export type Tuple<T, N extends number> = N extends N ? (number extends N ? T[] : _Tuple<T, N, []>) : never;
/**
 * Recursive type helper for constructing a fixed-length tuple of a given type.
 * This is utilized internally by Tuple to create the final fixed-length tuple.
 */
type _Tuple<T, N extends number, R extends unknown[]> = R['length'] extends N ? R : _Tuple<T, N, [T, ...R]>;

/**
 * Check an array size, and cast it to a tuple.
 * @param array - The array.
 * @param n - The size.
 * @returns The case tuple, or throws Error.
 */
export function assertLength<T, N extends number>(array: T[], n: N): Tuple<T, N> {
  if (array.length !== n) {
    throw new Error(`Wrong 'fixed array' size. Expected ${n}, got ${array.length}.`);
  }
  return array as Tuple<T, N>;
}
/**
 * Annoying, mapping a tuple does not preserve length.
 * This is a helper to preserve length during a map operation.
 * @typeparam T - The original array type.
 */
type MapTuple<T extends any[], F extends (item: any) => any> = {
  [K in keyof T]: T[K] extends infer U ? (F extends (item: U) => infer V ? V : never) : never;
};

/**
 * Annoyingly, mapping a tuple does not preserve length.
 * This is a helper to preserve length during a map operation.
 * @see https://github.com/microsoft/TypeScript/issues/29841.
 * @param array - A tuple array.
 */
export function mapTuple<T extends any[], F extends (item: any) => any>(tuple: T, fn: F): MapTuple<T, F> {
  return tuple.map(fn) as MapTuple<T, F>;
}
