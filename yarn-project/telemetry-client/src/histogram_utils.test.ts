import { exponentialBuckets, linearBuckets, millisecondBuckets } from './histogram_utils.js';

describe('linearBuckets', () => {
  it.each([[10, 1_000, 5, [10, 208, 406, 604, 802, 1000]]] as const)(
    'should return the expected buckets for a given range',
    (start, end, count, expected) => {
      expect(linearBuckets(start, end, count)).toEqual(expected);
    },
  );
});

describe('exponentialBuckets', () => {
  it.each([[2, 8, [1, 1.19, 1.41, 1.68, 2, 2.38, 2.83, 3.36, 4].map(x => expect.closeTo(x, 2))]] as const)(
    'should return the expected buckets for a given range',
    (scale, count, expected) => {
      expect(exponentialBuckets(scale, count)).toEqual(expected);
    },
  );
});

describe('millisecondBuckets', () => {
  it('should throw an error if significantFractionalDigits is less than 1', () => {
    expect(() => millisecondBuckets(0)).toThrow();
  });

  it('should return the expected buckets for milliseconds', () => {
    expect(millisecondBuckets(1, 16)).toEqual([
      10, // 2^0 * 10
      12,
      14,
      17,
      20, // 2^1 * 10
      24,
      28,
      34,
      40, // 2^2 * 10
      48,
      57,
      67,
      80, // 2^3 * 10
      95,
      113,
      135,
      160, // 2^4 * 10
    ]);
  });
});
