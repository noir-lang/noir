import { compact, mapValues } from './object.js';

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

describe('compact', () => {
  it('should remove keys with undefined values', () => {
    const obj = { a: 1, b: undefined, c: 3 };
    const result = compact(obj);
    expect(result).toEqual({ a: 1, c: 3 });
  });

  it('should not remove keys with falsey but not undefined values', () => {
    const obj = { a: false, b: 0, c: '', d: null, e: [] };
    const result = compact(obj);
    expect(result).toEqual(obj);
  });

  it('should handle an empty object', () => {
    const obj = {};
    const result = compact(obj);
    expect(result).toEqual({});
  });

  it('should handle an object with all undefined values', () => {
    const obj = { a: undefined, b: undefined, c: undefined };
    const result = compact(obj);
    expect(result).toEqual({});
  });

  it('should handle an object with no undefined values', () => {
    const obj = { a: 1, b: 2, c: 3 };
    const result = compact(obj);
    expect(result).toEqual({ a: 1, b: 2, c: 3 });
  });
});
