import { all, filter } from './index.js';

function* values(vals: number[] = [0, 1, 2, 3, 4]): Generator<number, void, undefined> {
  yield* vals;
}

async function* asyncValues(vals: number[] = [0, 1, 2, 3, 4]): AsyncGenerator<number, void, undefined> {
  yield* values(vals);
}

describe('filter iterable', () => {
  it('should filter all values greater than 2', () => {
    const res = all(filter(values(), val => val > 2));

    expect(res[Symbol.iterator]).toBeTruthy();
    expect(res).toEqual([3, 4]);
  });

  it('should filter all values less than 2', () => {
    const res = all(filter(values(), val => val < 2));

    expect(res[Symbol.iterator]).toBeTruthy();
    expect(res).toEqual([0, 1]);
  });

  it('should filter all values greater than 2 with a promise', () => {
    const res = all(filter(values(), val => val > 2));

    expect(res[Symbol.iterator]).toBeTruthy();
    expect(res).toEqual([3, 4]);
  });

  it('should filter all values greater than 2 with a promise', async () => {
    // eslint-disable-next-line require-await
    const res = filter(values(), async val => val > 2);

    expect(res[Symbol.asyncIterator]).toBeTruthy();
    await expect(all(res)).resolves.toEqual([3, 4]);
  });

  it('should filter all async values greater than 2', async () => {
    const res = filter(asyncValues(), val => val > 2);

    expect(res[Symbol.asyncIterator]).toBeTruthy();
    await expect(all(res)).resolves.toEqual([3, 4]);
  });

  it('should filter all async values greater than 2 with a promise', async () => {
    // eslint-disable-next-line require-await
    const res = filter(asyncValues(), async val => val > 2);

    expect(res[Symbol.asyncIterator]).toBeTruthy();
    await expect(all(res)).resolves.toEqual([3, 4]);
  });

  it('should filter values with indexes', () => {
    const vals = [4, 3, 2, 1, 0];
    const callbackArgs: any[] = [];
    const gen = filter(values(vals), (...args: any[]) => {
      callbackArgs.push(args);
      return true;
    });
    expect(gen[Symbol.iterator]).toBeTruthy();

    const results = all(gen);
    expect(results).toHaveLength(vals.length);
    expect(callbackArgs).toHaveLength(vals.length);

    vals.forEach((value, index) => {
      expect(callbackArgs[index][0]).toEqual(value);
      expect(callbackArgs[index][1]).toEqual(index);
    });
  });

  it('should filter async values with indexes', async () => {
    const vals = [4, 3, 2, 1, 0];
    const callbackArgs: any[] = [];
    const gen = filter(asyncValues(vals), (...args: any[]) => {
      callbackArgs.push(args);
      return true;
    });
    expect(gen[Symbol.asyncIterator]).toBeTruthy();

    const results = await all(gen);
    expect(results).toHaveLength(vals.length);
    expect(callbackArgs).toHaveLength(vals.length);

    vals.forEach((value, index) => {
      expect(callbackArgs[index][0]).toEqual(value);
      expect(callbackArgs[index][1]).toEqual(index);
    });
  });
});
