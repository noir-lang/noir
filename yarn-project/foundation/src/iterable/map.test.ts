import { all, map } from './index.js';

async function* asyncGenerator(vals: number[] = [1]): AsyncGenerator<number, void, undefined> {
  yield* vals;
}

function* generator(vals: number[] = [1]): Generator<number, void, undefined> {
  yield* vals;
}

async function* source(
  vals: number[] = [1],
): Generator<number, void, undefined> | AsyncGenerator<number, void, undefined> {
  yield* vals;
}

describe('map iterable', () => {
  it('should map an async generator', async () => {
    const gen = map(asyncGenerator(), val => val + 1);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(1);
    expect(results[0]).toEqual(2);
  });

  it('should map an async generator with indexes', async () => {
    const vals = [4, 3, 2, 1, 0];
    const gen = map(asyncGenerator(vals), (...args: any[]) => args);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(vals.length);

    vals.forEach((value, index) => {
      expect(results[index][0]).toEqual(value);
      expect(results[index][1]).toEqual(index);
    });
  });

  it('should map an async generator to a promise', async () => {
    const gen = map(asyncGenerator(), val => val + 1);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(1);
    expect(results[0]).toEqual(2);
  });

  it('should map an iterator', () => {
    const gen = map(generator(), val => val + 1);
    expect(gen[Symbol.iterator]).toBeTruthy();

    const results = all(gen);
    expect(results).toHaveLength(1);
    expect(results[0]).toEqual(2);
  });

  it('should map an iterator with indexes', () => {
    const vals = [4, 3, 2, 1, 0];
    const gen = map(generator(vals), (...args: any[]) => args);
    expect(gen[Symbol.iterator]).toBeTruthy();

    const results = all(gen);
    expect(results).toHaveLength(vals.length);

    vals.forEach((value, index) => {
      expect(results[index][0]).toEqual(value);
      expect(results[index][1]).toEqual(index);
    });
  });

  it('should map an iterator to a promise', async () => {
    // eslint-disable-next-line require-await
    const gen = map(generator(), async val => val + 1);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(1);
    expect(results[0]).toEqual(2);
  });

  it('should map a source', async () => {
    const gen = map(source(), val => val + 1);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(1);
    expect(results[0]).toEqual(2);
  });

  it('should map a source with indexes', async () => {
    const vals = [4, 3, 2, 1, 0];
    const gen = map(source(vals), (...args: any[]) => args);
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(vals.length);

    vals.forEach((value, index) => {
      expect(results[index][0]).toEqual(value);
      expect(results[index][1]).toEqual(index);
    });
  });
});
