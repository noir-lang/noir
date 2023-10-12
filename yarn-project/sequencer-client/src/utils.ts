/**
 * Returns a promise that resolves after ms milliseconds, returning "returnValue".
 * @param ms - How many milliseconds to sleep.
 * @param returnValue - The return value of the promise.
 */
export function sleep<T>(ms: number, returnValue: T): Promise<T> {
  return new Promise(resolve => setTimeout(() => resolve(returnValue), ms));
}

/**
 * Returns the lowest power of two that is greater of equal to the input.
 * @param num - The input.
 */
export function ceilPowerOfTwo(num: number): number {
  return 2 ** Math.ceil(Math.log2(num));
}
