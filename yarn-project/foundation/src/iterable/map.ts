import { isAsyncIterable } from './isAsyncIt.js';
import { peek } from './peek.js';

/**
 * Takes an (async) iterable and returns one with each item mapped by the passed
 * function.
 * @param source - The iterable to run the map function on.
 * @param func - The function to run over the iterable's items.
 * @returns A generator of the mapped items.
 */
function map<I, O>(
  source: Iterable<I>,
  func: (val: I, index: number) => Promise<O>,
): AsyncGenerator<O, void, undefined>;
function map<I, O>(source: Iterable<I>, func: (val: I, index: number) => O): Generator<O, void, undefined>;
function map<I, O>(
  source: AsyncIterable<I> | Iterable<I>,
  func: (val: I, index: number) => O | Promise<O>,
): AsyncGenerator<O, void, undefined>;
function map<I, O>(
  source: AsyncIterable<I> | Iterable<I>,
  func: (val: I, index: number) => O | Promise<O>,
): AsyncGenerator<O, void, undefined> | Generator<O, void, undefined> {
  let index = 0;

  if (isAsyncIterable(source)) {
    return (async function* () {
      for await (const val of source) {
        yield func(val, index++);
      }
    })();
  }

  // if mapping function returns a promise we have to return an async generator
  const peekable = peek(source);
  const { value, done } = peekable.next();

  if (done === true) {
    return (function* () {})();
  }

  const res = func(value, index++);

  // @ts-expect-error .then is not present on O
  if (typeof res.then === 'function') {
    return (async function* () {
      yield await res;

      for await (const val of peekable) {
        yield func(val, index++);
      }
    })();
  }

  const fn = func as (val: I, index: number) => O;

  return (function* () {
    yield res as O;

    for (const val of peekable) {
      yield fn(val, index++);
    }
  })();
}

export { map };
