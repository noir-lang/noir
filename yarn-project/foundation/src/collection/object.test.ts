import { mapValues } from './object.js';

describe('mapValues', () => {
  it('should return a new object with mapped values', () => {
    const obj = { a: 1, b: 2, c: 3 };
    const fn = (value: number) => value * 2;

    const result = mapValues(obj, fn);

    expect(result).toEqual({ a: 2, b: 4, c: 6 });
  });

  it('should handle an empty object', () => {
    const obj = {};
    const fn = (value: number) => value * 2;

    const result = mapValues(obj, fn);

    expect(result).toEqual({});
  });

  it('should handle different value types', () => {
    const obj = { a: 'hello', b: true, c: [1, 2, 3] };
    const fn = (value: any) => typeof value;

    const result = mapValues(obj, fn);

    expect(result).toEqual({ a: 'string', b: 'boolean', c: 'object' });
  });
});
