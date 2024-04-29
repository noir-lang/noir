import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

import { type IsEmpty, type Ordered } from '../interfaces/index.js';

// Define these utils here as their design is very specific to kernel's accumulated data and not general enough to be put in foundation.

// Returns number of non-empty items in an array.
export function countAccumulatedItems<T extends IsEmpty>(arr: T[]) {
  return arr.reduce((num, item, i) => {
    if (!item.isEmpty()) {
      if (num !== i) {
        throw new Error('Non-empty items must be placed continuously from index 0.');
      }
      return num + 1;
    }
    return num;
  }, 0);
}

// Merges two arrays of length N into an array of length N.
export function mergeAccumulatedData<T extends IsEmpty, N extends number>(
  arr0: Tuple<T, N>,
  arr1: Tuple<T, N>,
  length: N = arr0.length as N, // Need this for ts to infer the return Tuple length.
): Tuple<T, N> {
  const numNonEmptyItems0 = countAccumulatedItems(arr0);
  const numNonEmptyItems1 = countAccumulatedItems(arr1);
  if (numNonEmptyItems0 + numNonEmptyItems1 > length) {
    throw new Error('Combined non-empty items exceeded the maximum allowed.');
  }

  const arr = [...arr0] as Tuple<T, N>;
  arr1.slice(0, numNonEmptyItems1).forEach((item, i) => (arr[i + numNonEmptyItems0] = item));
  return arr;
}

// Sort items by their counters in ascending order. All empty items (counter === 0) are padded to the right.
export function sortByCounter<T extends Ordered & IsEmpty, N extends number>(arr: Tuple<T, N>): Tuple<T, N> {
  return [...arr].sort((a, b) => {
    if (a.counter === b.counter) {
      return 0;
    }
    if (a.isEmpty()) {
      return 1; // Move empty items to the right.
    }
    if (b.isEmpty()) {
      return -1; // Move non-empty items to the left.
    }
    return a.counter - b.counter;
  }) as Tuple<T, N>;
}

export function sortByCounterGetSortedHints<T extends Ordered & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  length: N = arr.length as N, // Need this for ts to infer the return Tuple length.
): [Tuple<T, N>, Tuple<number, N>] {
  const itemsWithIndexes = arr.map((item, i) => ({
    item,
    originalIndex: i,
    counter: item.counter,
    isEmpty: () => item.isEmpty(),
  }));
  const sorted = sortByCounter(itemsWithIndexes);
  const items = sorted.map(({ item }) => item) as Tuple<T, N>;

  const indexHints = makeTuple(length, () => 0);
  sorted.forEach(({ originalIndex }, i) => {
    if (!items[i].isEmpty()) {
      indexHints[originalIndex] = i;
    }
  });

  return [items, indexHints];
}

export function isEmptyArray<T extends IsEmpty>(arr: T[]): boolean {
  return arr.every(item => item.isEmpty());
}

export function getNonEmptyItems<T extends IsEmpty>(arr: T[]): T[] {
  return arr.filter(item => !item.isEmpty());
}
