export async function asyncMap<T, U>(arr: T[], fn: (e: T, i: number) => Promise<U>): Promise<U[]> {
  const results: U[] = [];
  for (let i = 0; i < arr.length; ++i) {
    results.push(await fn(arr[i], i));
  }
  return results;
}
