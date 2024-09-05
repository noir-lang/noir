/** Returns a new object with the same keys and where each value has been passed through the mapping function. */
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Record<K, T>,
  fn: (value: T, key: K) => U,
): Record<K, U>;
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Partial<Record<K, T>>,
  fn: (value: T, key: K) => U,
): Partial<Record<K, U>>;
export function mapValues<K extends string | number | symbol, T, U>(
  obj: Record<K, T>,
  fn: (value: T, key: K) => U,
): Record<K, U> {
  const result: Record<K, U> = {} as Record<K, U>;
  for (const key in obj) {
    result[key] = fn(obj[key], key);
  }
  return result;
}

/** Returns a new object where all keys with undefined values have been removed. */
export function compact<T extends object>(obj: T): { [P in keyof T]+?: Exclude<T[P], undefined> } {
  const result: any = {};
  for (const key in obj) {
    if (obj[key] !== undefined) {
      result[key] = obj[key];
    }
  }
  return result;
}
