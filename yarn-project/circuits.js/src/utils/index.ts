import { type Tuple } from '@aztec/foundation/serialize';

import type { IsEmpty, Ordered } from '../interfaces/index.js';

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

// Sort items by a provided compare function. All empty items are padded to the right.
function genericSort<T extends IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  compareFn: (a: T, b: T) => number,
  ascending: boolean = true,
): Tuple<T, N> {
  return [...arr].sort((a, b) => {
    if (a.isEmpty()) {
      return 1; // Move empty items to the right.
    }
    if (b.isEmpty()) {
      return -1; // Move non-empty items to the left.
    }
    return ascending ? compareFn(a, b) : compareFn(b, a);
  }) as Tuple<T, N>;
}

function compareByCounter<T extends Ordered>(a: T, b: T): number {
  return a.counter - b.counter;
}

export function sortByCounter<T extends Ordered & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  ascending: boolean = true,
): Tuple<T, N> {
  return genericSort(arr, compareByCounter, ascending);
}

export function isEmptyArray<T extends IsEmpty>(arr: T[]): boolean {
  return arr.every(item => item.isEmpty());
}

export function getNonEmptyItems<T extends IsEmpty>(arr: T[]): T[] {
  return arr.filter(item => !item.isEmpty());
}
