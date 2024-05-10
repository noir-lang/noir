import { all } from './index.js';
import { type CompareFunction, sort } from './index.js';

describe('sort iterable', () => {
  it('should sort all entries of an iterator', () => {
    const values = ['foo', 'bar'];
    const sorter: CompareFunction<string> = (a, b) => {
      return a.localeCompare(b);
    };

    const gen = sort(values, sorter);
    expect(gen[Symbol.iterator]).toBeTruthy();

    const res = all(gen);
    expect(res).toEqual(['bar', 'foo']);
  });

  it('should sort all entries of an async iterator', async () => {
    const values = async function* (): AsyncGenerator<string, void, undefined> {
      yield* ['foo', 'bar'];
    };
    const sorter: CompareFunction<string> = (a, b) => {
      return a.localeCompare(b);
    };

    const gen = sort(values(), sorter);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const res = await all(gen);
    expect(res).toEqual(['bar', 'foo']);
  });
});
