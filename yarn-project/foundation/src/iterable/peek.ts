export interface Peek<T> {
  peek(): IteratorResult<T, undefined>;
}

export interface AsyncPeek<T> {
  peek(): Promise<IteratorResult<T, undefined>>;
}

export interface Push<T> {
  push(value: T): void;
}

export type Peekable<T> = Iterable<T> & Peek<T> & Push<T> & Iterator<T>;

export type AsyncPeekable<T> = AsyncIterable<T> & AsyncPeek<T> & Push<T> & AsyncIterator<T>;

/**
 * Utility function that allows peeking into the contents of an async iterator.
 * @param iterable - The async iterator to peek the values of.
 */
function peekable<T>(iterable: Iterable<T>): Peekable<T>;
function peekable<T>(iterable: AsyncIterable<T>): AsyncPeekable<T>;
function peekable<T>(iterable: Iterable<T> | AsyncIterable<T>): Peekable<T> | AsyncPeekable<T> {
  const [iterator, symbol] =
    // @ts-expect-error can't use Symbol.asyncIterator to index iterable since it might be Iterable
    iterable[Symbol.asyncIterator] != null
      ? // @ts-expect-error can't use Symbol.asyncIterator to index iterable since it might be Iterable
        [iterable[Symbol.asyncIterator](), Symbol.asyncIterator]
      : // @ts-expect-error can't use Symbol.iterator to index iterable since it might be AsyncIterable
        [iterable[Symbol.iterator](), Symbol.iterator];

  const queue: any[] = [];

  // @ts-expect-error can't use symbol to index peekable
  return {
    peek: () => {
      return iterator.next();
    },
    push: (value: any) => {
      queue.push(value);
    },
    next: () => {
      if (queue.length > 0) {
        return {
          done: false,
          value: queue.shift(),
        };
      }

      return iterator.next();
    },
    [symbol]() {
      return this;
    },
  };
}

export { peekable as peek };
