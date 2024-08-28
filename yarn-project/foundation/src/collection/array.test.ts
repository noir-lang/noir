import { compactArray, removeArrayPaddingEnd, times, unique } from './array.js';

describe('times', () => {
  it('should return an array with the result from all executions', () => {
    const result = times(5, i => i * 2);
    expect(result).toEqual([0, 2, 4, 6, 8]);
  });

  it('should return an empty array when n is 0', () => {
    const result = times(0, i => i * 2);
    expect(result).toEqual([]);
  });
});

describe('removeArrayPaddingEnd', () => {
  it('removes padding from the end of the array', () => {
    expect(removeArrayPaddingEnd([0, 1, 2, 0, 3, 4, 0, 0], i => i === 0)).toEqual([0, 1, 2, 0, 3, 4]);
  });

  it('does not change array if no padding', () => {
    expect(removeArrayPaddingEnd([0, 1, 2, 0, 3, 4], i => i === 0)).toEqual([0, 1, 2, 0, 3, 4]);
  });

  it('handles no empty items ', () => {
    expect(removeArrayPaddingEnd([1, 2, 3, 4], i => i === 0)).toEqual([1, 2, 3, 4]);
  });

  it('handles empty array', () => {
    expect(removeArrayPaddingEnd([], i => i === 0)).toEqual([]);
  });

  it('handles array with empty items', () => {
    expect(removeArrayPaddingEnd([0, 0, 0], i => i === 0)).toEqual([]);
  });
});

describe('compactArray', () => {
  it('works as expected', () => {
    expect(compactArray([3, undefined, 4, undefined])).toEqual([3, 4]);
  });

  it('handles an empty array', () => {
    expect(compactArray([])).toEqual([]);
  });

  it('handles an array with just undefineds', () => {
    expect(compactArray([undefined, undefined])).toEqual([]);
  });

  it('handles an array with no undefineds', () => {
    expect(compactArray([2, 3])).toEqual([2, 3]);
  });

  it('does not remove falsey values', () => {
    expect(compactArray([0, null, false, '', [], undefined])).toEqual([0, null, false, '', []]);
  });
});

describe('unique', () => {
  it('works with bigints', () => {
    expect(unique([1n, 2n, 1n])).toEqual([1n, 2n]);
  });
});
