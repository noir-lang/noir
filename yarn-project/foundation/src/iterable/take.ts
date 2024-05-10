import { isAsyncIterable } from './isAsyncIt.js';

/**
 * Stop iteration after n items have been received.
 * @param source - An iterable to take n items from.
 * @param limit - The number of items to take from the iterable.
 * @returns A generator, limited to n items.
 */
function take<T>(source: Iterable<T>, limit: number): Generator<T, void, undefined>;
function take<T>(source: Iterable<T> | AsyncIterable<T>, limit: number): AsyncGenerator<T, void, undefined>;
function take<T>(
  source: Iterable<T> | AsyncIterable<T>,
  limit: number,
): AsyncGenerator<T, void, undefined> | Generator<T, void, undefined> {
  if (isAsyncIterable(source)) {
    return (async function* () {
      let items = 0;

      if (limit < 1) {
        return;
      }

      for await (const entry of source) {
        yield entry;

        items++;

        if (items === limit) {
          return;
        }
      }
    })();
  }

  return (function* () {
    let items = 0;

    if (limit < 1) {
      return;
    }

    for (const entry of source) {
      yield entry;

      items++;

      if (items === limit) {
        return;
      }
    }
  })();
}

export { take };
