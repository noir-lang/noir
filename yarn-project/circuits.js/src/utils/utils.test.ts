import { makeTuple } from '@aztec/foundation/array';
import { type IsEmpty } from '@aztec/foundation/interfaces';
import { type Tuple } from '@aztec/foundation/serialize';

import { concatAccumulatedData, countAccumulatedItems, mergeAccumulatedData } from './index.js';

class TestItem {
  constructor(public value: number) {}

  static empty() {
    return new TestItem(0);
  }

  isEmpty() {
    return this.value === 0;
  }
}

describe('hints utils', () => {
  const expectEmptyArrays = (arr: IsEmpty[]) => {
    arr.forEach(item => expect(item.isEmpty()).toBe(true));
  };

  describe('countAccumulatedItems', () => {
    it('counts the number of non-empty items', () => {
      const arr = makeTuple(20, TestItem.empty);
      const num = 6;
      for (let i = 0; i < num; ++i) {
        arr[i] = new TestItem(i + 1);
      }
      expect(countAccumulatedItems(arr)).toBe(num);
    });

    it('throws if arr contains non-continuous non-empty items', () => {
      const arr = makeTuple(20, TestItem.empty);
      arr[1] = new TestItem(123);
      expect(() => countAccumulatedItems(arr)).toThrow('Non-empty items must be placed continuously from index 0.');
    });
  });

  describe('mergeAccumulatedData', () => {
    const length = 5;
    let arr0: Tuple<TestItem, typeof length>;
    let arr1: Tuple<TestItem, typeof length>;

    beforeEach(() => {
      arr0 = makeTuple(length, TestItem.empty);
      arr1 = makeTuple(length, TestItem.empty);
    });

    it('propagates items from arr0', () => {
      arr0[0] = new TestItem(12);
      arr0[1] = new TestItem(34);
      const res = mergeAccumulatedData(length, arr0, arr1);
      expect(res.slice(0, 2)).toEqual([arr0[0], arr0[1]]);
      expectEmptyArrays(res.slice(2));
    });

    it('propagates items from arr1', () => {
      arr1[0] = new TestItem(1);
      arr1[1] = new TestItem(2);
      const res = mergeAccumulatedData(length, arr0, arr1);
      expect(res.slice(0, 2)).toEqual([arr1[0], arr1[1]]);
      expectEmptyArrays(res.slice(2));
    });

    it('merges items from both arrays', () => {
      arr0[0] = new TestItem(12);
      arr0[1] = new TestItem(34);
      arr1[0] = new TestItem(1);
      arr1[1] = new TestItem(2);
      const res = mergeAccumulatedData(length, arr0, arr1);
      expect(res.slice(0, 4)).toEqual([arr0[0], arr0[1], arr1[0], arr1[1]]);
      expectEmptyArrays(res.slice(4));
    });

    it('throws if arr0 contains non-continuous items', () => {
      arr0[0] = new TestItem(12);
      arr0[2] = new TestItem(34);
      expect(() => mergeAccumulatedData(length, arr0, arr1)).toThrow(
        'Non-empty items must be placed continuously from index 0.',
      );
    });

    it('throws if arr1 contains non-continuous items', () => {
      arr1[0] = new TestItem(12);
      arr1[2] = new TestItem(34);
      expect(() => mergeAccumulatedData(length, arr0, arr1)).toThrow(
        'Non-empty items must be placed continuously from index 0.',
      );
    });

    it('throws if total number of items exceeds limit', () => {
      for (let i = 0; i < length; ++i) {
        arr0[i] = new TestItem(i + 1);
      }
      expect(mergeAccumulatedData(length, arr0, arr1)).toBeDefined();

      arr1[0] = new TestItem(1234);
      expect(() => mergeAccumulatedData(length, arr0, arr1)).toThrow(
        'Combined non-empty items exceeded the maximum allowed.',
      );
    });
  });

  describe('concatAccumulatedData', () => {
    const length0 = 3;
    const length1 = 5;
    const length = length0 + length1;
    let arr0: Tuple<TestItem, typeof length0>;
    let arr1: Tuple<TestItem, typeof length1>;

    beforeEach(() => {
      arr0 = makeTuple(length0, TestItem.empty);
      arr1 = makeTuple(length1, TestItem.empty);
    });

    it('propagates items from arr0', () => {
      arr0[0] = new TestItem(12);
      arr0[1] = new TestItem(34);
      const nullifiers = concatAccumulatedData(length, arr0, arr1);
      expect(nullifiers.slice(0, 2)).toEqual([arr0[0], arr0[1]]);
      expectEmptyArrays(nullifiers.slice(2));
    });

    it('propagates items from arr1', () => {
      arr1[0] = new TestItem(1);
      arr1[1] = new TestItem(2);
      const nullifiers = concatAccumulatedData(length, arr0, arr1);
      expect(nullifiers.slice(0, 2)).toEqual([arr1[0], arr1[1]]);
      expectEmptyArrays(nullifiers.slice(2));
    });

    it('combines items from both arrays', () => {
      arr0[0] = new TestItem(12);
      arr0[1] = new TestItem(34);
      arr1[0] = new TestItem(1);
      arr1[1] = new TestItem(2);
      const nullifiers = concatAccumulatedData(length, arr0, arr1);
      expect(nullifiers.slice(0, 4)).toEqual([arr0[0], arr0[1], arr1[0], arr1[1]]);
      expectEmptyArrays(nullifiers.slice(4));
    });

    it('combines all items from both arrays', () => {
      arr0 = makeTuple(length0, i => new TestItem(i + 1));
      arr1 = makeTuple(length1, i => new TestItem(i + 999));
      const nullifiers = concatAccumulatedData(length, arr0, arr1);
      expect(nullifiers).toEqual([...arr0, ...arr1]);
    });

    it('throws if given length is incorrect', () => {
      expect(() => concatAccumulatedData(length + 1, arr0, arr1)).toThrow(
        /Provided length does not match combined length./,
      );
    });

    it('throws if arr0 contains non-continuous items', () => {
      arr0[0] = new TestItem(12);
      arr0[2] = new TestItem(34);
      expect(() => concatAccumulatedData(length, arr0, arr1)).toThrow(
        'Non-empty items must be placed continuously from index 0.',
      );
    });

    it('throws if arr1 contains non-continuous items', () => {
      arr1[0] = new TestItem(12);
      arr1[2] = new TestItem(34);
      expect(() => concatAccumulatedData(length, arr0, arr1)).toThrow(
        'Non-empty items must be placed continuously from index 0.',
      );
    });
  });
});
