/**
 * Utility function to type check an AsyncIterable
 * @param thing - Input to type check
 * @returns Type-checked input
 */
export function isAsyncIterable<T>(thing: any): thing is AsyncIterable<T> {
  return thing[Symbol.asyncIterator] != null;
}
