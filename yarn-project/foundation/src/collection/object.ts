/** Returns a new object with the same keys and where each value has been passed through the mapping function. */
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Record<K, T>,
  fn: (value: T) => U,
): Record<K, U>;
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Partial<Record<K, T>>,
  fn: (value: T) => U,
): Partial<Record<K, U>>;
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Record<K, T>,
  fn: (value: T) => U,
): Record<K, U> {
  const result: Record<K, U> = {} as Record<K, U>;
  for (const key in obj) {
    result[key] = fn(obj[key]);
  }
  return result;
}
