import { randomBytes } from '@aztec/foundation/crypto';

import { type Database, open } from 'lmdb';

import { LmdbAztecCounter } from './counter.js';

describe('LmdbAztecCounter', () => {
  let db: Database;

  beforeEach(() => {
    db = open({} as any);
  });

  describe.each([
    ['floating point number', () => Math.random()],
    ['integers', () => (Math.random() * 1000) | 0],
    ['strings', () => randomBytes(8).toString('hex')],
    ['strings', () => [Math.random(), randomBytes(8).toString('hex')]],
  ])('counts occurrences of %s values', (_, genKey) => {
    let counter: LmdbAztecCounter<ReturnType<typeof genKey>>;
    beforeEach(() => {
      counter = new LmdbAztecCounter(db, 'test');
    });

    it('returns 0 for unknown keys', () => {
      expect(counter.get(genKey())).toEqual(0);
    });

    it('increments values', async () => {
      const key = genKey();
      await counter.update(key, 1);

      expect(counter.get(key)).toEqual(1);
    });

    it('decrements values', async () => {
      const key = genKey();
      await counter.update(key, 1);
      await counter.update(key, -1);

      expect(counter.get(key)).toEqual(0);
    });

    it('throws when decrementing below zero', async () => {
      const key = genKey();
      await counter.update(key, 1);

      await expect(counter.update(key, -2)).rejects.toThrow();
    });

    it('increments values by a delta', async () => {
      const key = genKey();
      await counter.update(key, 1);
      await counter.update(key, 2);

      expect(counter.get(key)).toEqual(3);
    });

    it('resets the counter', async () => {
      const key = genKey();
      await counter.update(key, 1);
      await counter.update(key, 2);
      await counter.set(key, 0);

      expect(counter.get(key)).toEqual(0);
    });

    it('iterates over entries', async () => {
      const key = genKey();
      await counter.update(key, 1);
      await counter.update(key, 2);

      expect([...counter.entries()]).toEqual([[key, 3]]);
    });
  });

  it.each([
    [
      [
        ['c', 2342],
        ['a', 8],
        ['b', 1],
      ],
      [
        ['a', 8],
        ['b', 1],
        ['c', 2342],
      ],
    ],
    [
      [
        [10, 2],
        [18, 1],
        [1, 2],
      ],
      [
        [1, 2],
        [10, 2],
        [18, 1],
      ],
    ],
    [
      [
        [[10, 'a'], 1],
        [[10, 'c'], 2],
        [[11, 'b'], 1],
        [[9, 'f'], 1],
        [[10, 'b'], 1],
      ],
      [
        [[9, 'f'], 1],
        [[10, 'a'], 1],
        [[10, 'b'], 1],
        [[10, 'c'], 2],
        [[11, 'b'], 1],
      ],
    ],
  ])('iterates in key order', async (insertOrder, expectedOrder) => {
    const counter = new LmdbAztecCounter(db, 'test');
    await Promise.all(insertOrder.map(([key, value]) => counter.update(key, value as number)));
    expect([...counter.entries()]).toEqual(expectedOrder);
  });
});
