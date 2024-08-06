/**
 * Creates a set of buckets that follow linear growth
 * @param start - The start of the range
 * @param end - The end of the range
 * @param count - The number of buckets
 * @returns - An array of bucket boundaries
 */
export function linearBuckets(start: number, end: number, count: number): number[] {
  const buckets = [];
  const step = (end - start) / count;
  for (let i = 0; i <= count; i++) {
    buckets.push(start + i * step);
  }
  return buckets;
}

/**
 * Creates an array of exponential buckets that follow the formulas at
 * https://opentelemetry.io/docs/specs/otel/metrics/data-model/#exponential-buckets
 *
 * The formula followed is: bucket[i] = base ** i, where base = 2 ** (2 ** -scale). Naturally base will be very small when scale > 0.
 * This ensures that between each power of 2, there are 2 ** scale intermediate buckets.
 *
 * @example
 * scale = 2, count = 8
 * base = 2 ** (2 ** -2) = 1.189207115
 * |bucket index| value |
 * |------------|-------|
 * |     0      | 1     |
 * |     1      | 1.18  |
 * |     2      | 1.41  |
 * |     3      | 1.68  |
 * |     4      | 2     |
 * |     5      | 2.37  |
 * |     6      | 2.82  |
 * |     7      | 3.36  |
 * |     8      | 4     |
 *
 * @param scale - The "precision" of the buckets. The higher the scale, the more buckets will be created
 * @param count - The total number of buckets
 * @returns - An array of bucket boundaries
 */
export function exponentialBuckets(scale: number, count: number): number[] {
  const buckets: number[] = [];
  const base = 2 ** (2 ** -scale);
  for (let i = 0; i <= count; i++) {
    buckets.push(base ** i);
  }
  return buckets;
}

/**
 * Creates an array of exponential buckets optimized for milliseconds
 * @param significantFractionalDigits - The number of significant digits to round to
 * @param count - The number of buckets. Defaults to 60
 * @returns - An array of bucket boundaries
 */
export function millisecondBuckets(significantFractionalDigits: number, count = 60): number[] {
  if (significantFractionalDigits < 1) {
    // if significant digits is 1 then we end up having duplicate buckets
    throw new Error('significantFractionalDigits must be >= 1');
  }

  const scale = 10 ** significantFractionalDigits;
  return exponentialBuckets(2, count).map(x => Math.round(x * scale));
}
