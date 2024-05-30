import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

import type { IsEmpty, Ordered, Positioned } from '../interfaces/index.js';

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
export function genericSort<T extends IsEmpty, N extends number>(
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

export function compareByCounter<T extends Ordered>(a: T, b: T): number {
  return a.counter - b.counter;
}

export function sortByCounter<T extends Ordered & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  ascending: boolean = true,
): Tuple<T, N> {
  return genericSort(arr, compareByCounter, ascending);
}

export function compareByPositionThenCounter<T extends Ordered & Positioned>(a: T, b: T): number {
  const positionComp = a.position.cmp(b.position);
  if (positionComp !== 0) {
    return positionComp;
  }
  return a.counter - b.counter;
}

export function sortByPositionThenCounter<T extends Ordered & Positioned & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  ascending: boolean = true,
): Tuple<T, N> {
  return genericSort(arr, compareByPositionThenCounter, ascending);
}

export interface SortOptions {
  ascending: boolean;
  // If you're using this in the circuits, and checking via `assert_sorted`, then you should use 'sorted'.
  // If you're using this in the circuits, and checking via `check_permutation`, then you should use 'original'.
  hintIndexesBy: 'original' | 'sorted';
}

const defaultSortOptions: SortOptions = {
  ascending: true,
  hintIndexesBy: 'sorted',
};

export function sortAndGetSortedHints<T extends IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  compareFn: (a: T, b: T) => number,
  length: N = arr.length as N, // Need this for ts to infer the return Tuple length.
  { ascending = true, hintIndexesBy = 'sorted' }: SortOptions,
): [Tuple<T, N>, Tuple<number, N>] {
  const itemsWithIndexes = arr.map((item, i) => ({
    item,
    originalIndex: i,
    isEmpty: () => item.isEmpty(),
  }));

  const sorted = genericSort(itemsWithIndexes, (a, b) => compareFn(a.item, b.item), ascending);
  const items = sorted.map(({ item }) => item) as Tuple<T, N>;

  const indexHints = makeTuple(length, () => 0);
  if (hintIndexesBy === 'sorted') {
    sorted.forEach(({ originalIndex }, i) => {
      if (!items[i].isEmpty()) {
        indexHints[originalIndex] = i;
      }
    });
  } else {
    sorted.forEach(({ originalIndex }, i) => {
      if (!items[i].isEmpty()) {
        indexHints[i] = originalIndex;
      }
    });
  }

  return [items, indexHints];
}

export function sortByCounterGetSortedHints<T extends Ordered & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  length: N = arr.length as N, // Need this for ts to infer the return Tuple length.
  options: SortOptions = defaultSortOptions,
): [Tuple<T, N>, Tuple<number, N>] {
  return sortAndGetSortedHints(arr, compareByCounter, length, options);
}

export function sortByPositionThenCounterGetSortedHints<T extends Ordered & Positioned & IsEmpty, N extends number>(
  arr: Tuple<T, N>,
  length: N = arr.length as N, // Need this for ts to infer the return Tuple length.
  options: SortOptions = defaultSortOptions,
): [Tuple<T, N>, Tuple<number, N>] {
  return sortAndGetSortedHints(arr, compareByPositionThenCounter, length, options);
}

/**
 * @param arr An array sorted on position then counter.
 * @param length for type inference.
 * @param getEmptyItem helper function to get an empty item.
 * @returns the array deduplicated by position, and the original run lengths of each position.
 */
export function deduplicateSortedArray<T extends Ordered & IsEmpty & Positioned, N extends number>(
  arr: Tuple<T, N>,
  length: N = arr.length as N,
  getEmptyItem: () => T,
): [Tuple<T, N>, Tuple<number, N>] {
  const dedupedArray = makeTuple(length, getEmptyItem) as Tuple<T, N>;
  const runLengths = makeTuple(length, () => 0);

  let dedupedIndex = 0;
  let runCounter = 0;
  let currentPosition = arr[0].position;

  let i = 0;
  for (; i < length; i++) {
    const item = arr[i];

    if (item.isEmpty()) {
      break; // Stop processing when encountering the first empty item.
    }

    if (item.position.equals(currentPosition)) {
      runCounter++;
    } else {
      dedupedArray[dedupedIndex] = arr[i - 1];
      runLengths[dedupedIndex] = runCounter;
      dedupedIndex++;
      runCounter = 1;
      currentPosition = item.position;
    }
  }

  if (runCounter > 0) {
    dedupedArray[dedupedIndex] = arr[i - 1];
    runLengths[dedupedIndex] = runCounter;
    dedupedIndex++;
  }

  // Fill the remaining part of the deduped array and run lengths with empty items and zeros.
  for (let i = dedupedIndex; i < length; i++) {
    dedupedArray[i] = getEmptyItem();
    runLengths[i] = 0;
  }

  return [dedupedArray, runLengths];
}

export function isEmptyArray<T extends IsEmpty>(arr: T[]): boolean {
  return arr.every(item => item.isEmpty());
}

export function getNonEmptyItems<T extends IsEmpty>(arr: T[]): T[] {
  return arr.filter(item => !item.isEmpty());
}
