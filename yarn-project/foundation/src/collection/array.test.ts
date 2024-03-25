import { removeArrayPaddingEnd, times } from './array.js';

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
