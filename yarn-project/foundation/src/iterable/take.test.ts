import { all, take } from './index.js';

describe('take from iterable', () => {
  it('should limit the number of values returned from an iterable', () => {
    const values = [0, 1, 2, 3, 4];

    const gen = take(values, 2);
    expect(gen[Symbol.iterator]).toBeTruthy();

    const res = all(gen);
    expect(res).toEqual([0, 1]);
  });

  it('should limit the number of values returned from an async iterable', async () => {
    const values = async function* (): AsyncGenerator<number, void, undefined> {
      yield* [0, 1, 2, 3, 4];
    };

    const gen = take(values(), 2);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const res = await all(gen);
    expect(res).toEqual([0, 1]);
  });
});
