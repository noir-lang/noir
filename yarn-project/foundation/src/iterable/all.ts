import { isAsyncIterable } from './isAsyncIt.js';

/**
 * Collects all values from an (async) iterable and returns them as an array
 * @param source - Iterable to collect all values from
 * @returns All of the iterable's values as an array.
 */
function all<T>(source: Iterable<T>): T[];
function all<T>(source: Iterable<T> | AsyncIterable<T>): Promise<T[]>;
function all<T>(source: Iterable<T> | AsyncIterable<T>): Promise<T[]> | T[] {
  if (isAsyncIterable(source)) {
    return (async () => {
      const arr = [];

      for await (const entry of source) {
        arr.push(entry);
      }

      return arr;
    })();
  }

  const arr = [];

  for (const entry of source) {
    arr.push(entry);
  }

  return arr;
}

export { all };
