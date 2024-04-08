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
  _length: N,
  arr0: Tuple<T, N>,
  arr1: Tuple<T, N>,
): Tuple<T, N> {
  const numNonEmptyItems0 = countAccumulatedItems(arr0);
  const numNonEmptyItems1 = countAccumulatedItems(arr1);
  if (numNonEmptyItems0 + numNonEmptyItems1 > arr0.length) {
    throw new Error('Combined non-empty items exceeded the maximum allowed.');
  }

  const arr = [...arr0] as Tuple<T, N>;
  arr1.slice(0, numNonEmptyItems1).forEach((item, i) => (arr[i + numNonEmptyItems0] = item));
  return arr;
}

// Sort items by their counters in ascending order. All empty items (counter === 0) are padded to the right.
export function sortByCounter<T extends Ordered>(arr: T[]): T[] {
  return [...arr].sort((a, b) => {
    if (a.counter === b.counter) {
      return 0;
    }
    if (a.counter === 0) {
      return 1; // Move empty items to the right.
    }
    if (b.counter === 0) {
      return -1; // Move non-empty items to the left.
    }
    return a.counter - b.counter;
  });
}
