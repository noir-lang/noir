import { all } from './index.js';

describe('all iterable', () => {
  it('should collect all entries of an iterator as an array', () => {
    const values = [0, 1, 2, 3, 4];

    const res = all(values);

    expect(res).not.toHaveProperty('then');
    expect(res).toEqual(values);
  });

  it('should collect all entries of an async iterator as an array', async () => {
    const values = [0, 1, 2, 3, 4];

    const generator = (async function* (): AsyncGenerator<number, void, undefined> {
      yield* [0, 1, 2, 3, 4];
    })();

    const p = all(generator);
    expect(p).toHaveProperty('then');
    expect(p.then).toBeInstanceOf(Function);

    const res = await p;
    expect(res).toEqual(values);
  });
});
