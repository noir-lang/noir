import { randomBytes } from '@aztec/foundation/crypto';
import { all } from '@aztec/foundation/iterable';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';

import {
  type Datastore,
  Key,
  type KeyQueryFilter,
  type KeyQueryOrder,
  type Pair,
  type QueryFilter,
  type QueryOrder,
} from 'interface-datastore';
import drain from 'it-drain';
import length from 'it-length';
import { fromString as uint8ArrayFromString } from 'uint8arrays/from-string';

import { AztecDatastore } from './data_store.js';

describe('AztecDatastore with AztecLmdbStore', () => {
  let datastore: AztecDatastore;
  let aztecStore: AztecLmdbStore;

  beforeAll(() => {
    aztecStore = AztecLmdbStore.open();
  });

  beforeEach(async () => {
    datastore = new AztecDatastore(aztecStore);
    await aztecStore.clear();
  });

  it('should store and retrieve an item', async () => {
    const key = new Key('testKey');
    const value = new Uint8Array([1, 2, 3]);

    await datastore.put(key, value);
    const retrieved = datastore.get(key);

    expect(retrieved).toEqual(value);
  });

  it('should delete an item', async () => {
    const key = new Key('testKey');
    await datastore.put(key, new Uint8Array([1, 2, 3]));
    await datastore.delete(key);

    try {
      datastore.get(key);
    } catch (err) {
      expect(err).toHaveProperty('code', 'ERR_NOT_FOUND');
    }
  });

  it('batch operations commit correctly', async () => {
    const batch = datastore.batch();
    const key1 = new Key('key1');
    const key2 = new Key('key2');
    const value1 = new Uint8Array([1, 2, 3]);
    const value2 = new Uint8Array([4, 5, 6]);

    batch.put(key1, value1);
    batch.put(key2, value2);
    batch.delete(key1);
    await batch.commit();

    try {
      datastore.get(key1); // key1 should be deleted
    } catch (err) {
      expect(err).toHaveProperty('code', 'ERR_NOT_FOUND');
    }
    const retrieved2 = datastore.get(key2);

    expect(retrieved2.toString()).toEqual(value2.toString()); // key2 should exist
  });

  it('query data by prefix', async () => {
    await datastore.put(new Key('/prefix/123'), new Uint8Array([1, 2, 3]));
    await datastore.put(new Key('/prefix/456'), new Uint8Array([4, 5, 6]));
    await datastore.put(new Key('/foobar/789'), new Uint8Array([7, 8, 9]));

    const query = {
      prefix: '/prefix',
      limit: 2,
    };

    const results = [];
    for await (const item of datastore.query(query)) {
      results.push(item);
    }

    expect(results.length).toBe(2);
    expect(results.every(item => item.key.toString().startsWith(`${query.prefix}`))).toBeTruthy();
  });

  it('handle limits and offsets in queries', async () => {
    await datastore.put(new Key('item1'), new Uint8Array([1]));
    await datastore.put(new Key('item2'), new Uint8Array([2]));
    await datastore.put(new Key('item3'), new Uint8Array([3]));
    await datastore.put(new Key('item4'), new Uint8Array([4]));

    const query = {
      limit: 2,
      offset: 1,
    };

    const results = [];
    for await (const item of datastore.query(query)) {
      results.push(item);
    }

    expect(results.length).toBe(2);
    expect(results[0].key.toString()).toBe('/item2');
    expect(results[1].key.toString()).toBe('/item3');
  });

  it('memory map prunes correctly when limit is exceeded', async () => {
    // Insert more items than the memory limit to force pruning
    for (let i = 0; i < 10; i++) {
      await datastore.put(new Key(`key${i}`), new Uint8Array([i]));
    }

    // Check that data remains accessible even if it's no longer in the memory map
    for (let i = 0; i < 10; i++) {
      const result = datastore.get(new Key(`key${i}`));
      expect(result).toEqual(new Uint8Array([i]));
    }
  });

  it('data consistency with transitions between memory and database', async () => {
    for (let i = 0; i < 20; i++) {
      await datastore.put(new Key(`key${i}`), new Uint8Array([i]));
    }

    // Check data consistency
    for (let i = 0; i < 20; i++) {
      const value = datastore.get(new Key(`key${i}`));
      expect(value).toEqual(new Uint8Array([i]));
    }
  });

  describe('interface-datastore compliance tests', () => {
    interfaceDatastoreTests({
      setup() {
        const _aztecStore = AztecLmdbStore.open();
        const _datastore = new AztecDatastore(_aztecStore);
        // await _aztecStore.clear();
        return _datastore;
      },
      async teardown(store) {
        await all(store.deleteMany(store.queryKeys({})));
      },
    });
  });
});

export interface InterfaceDatastoreTest<D extends Datastore = Datastore> {
  setup(): D | Promise<D>;
  teardown(store: D): void | Promise<void>;
}

export function interfaceDatastoreTests<D extends Datastore = Datastore>(test: InterfaceDatastoreTest<D>): void {
  const cleanup = async (store: D): Promise<void> => {
    await test.teardown(store);
  };

  const createStore = async (): Promise<D> => {
    return await test.setup();
  };

  describe('put', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('simple', async () => {
      const k = new Key('/z/key');
      const v = uint8ArrayFromString('one');
      await store.put(k, v);

      expect(store.get(k)).toEqual(v);
    });

    it('parallel', async () => {
      const data: Pair[] = [];
      for (let i = 0; i < 52; i++) {
        data.push({ key: new Key(`/z/key${i}`), value: uint8ArrayFromString(`data${i}`) });
      }

      await Promise.all(
        data.map(async d => {
          await store.put(d.key, d.value);
        }),
      );

      const res = await all(store.getMany(data.map(d => d.key)));
      expect(res).toEqual(data);
    });
  });

  describe('putMany', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('streaming', async () => {
      const data: Pair[] = [];
      for (let i = 0; i < 100; i++) {
        data.push({ key: new Key(`/z/key${i}`), value: uint8ArrayFromString(`data${i}`) });
      }

      let index = 0;

      for await (const key of store.putMany(data)) {
        expect(data[index].key).toEqual(key);
        index++;
      }

      expect(index).toEqual(data.length);

      const res = await all(store.getMany(data.map(d => d.key)));
      expect(res).toEqual(data);
    });
  });

  describe('get', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('simple', async () => {
      const k = new Key('/z/one');
      await store.put(k, uint8ArrayFromString('hello'));
      const res = await store.get(k);
      expect(res).toEqual(uint8ArrayFromString('hello'));
    });

    it('should throw error for missing key', async () => {
      const k = new Key('/does/not/exist');

      try {
        await store.get(k);
      } catch (err) {
        expect(err).toHaveProperty('code', 'ERR_NOT_FOUND');
        return;
      }
    });
  });

  describe('getMany', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('streaming', async () => {
      const k = new Key('/z/one');
      await store.put(k, uint8ArrayFromString('hello'));
      const source = [k];

      const res = await all(store.getMany(source));
      expect(res).toHaveLength(1);
      expect(res[0].key).toEqual(k);
      expect(res[0].value).toEqual(uint8ArrayFromString('hello'));
    });

    it('should throw error for missing key', async () => {
      const k = new Key('/does/not/exist');

      try {
        await drain(store.getMany([k]));
      } catch (err) {
        expect(err).toHaveProperty('code', 'ERR_NOT_FOUND');
        return;
      }
    });
  });

  describe('delete', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    }, 10_000);

    it('simple', async () => {
      const k = new Key('/z/one');
      await store.put(k, uint8ArrayFromString('hello'));
      await store.get(k);
      await store.delete(k);
      const exists = await store.has(k);
      expect(exists).toEqual(false);
    });

    it('parallel', async () => {
      const data: Array<[Key, Uint8Array]> = [];
      for (let i = 0; i < 100; i++) {
        data.push([new Key(`/a/key${i}`), uint8ArrayFromString(`data${i}`)]);
      }

      await Promise.all(
        data.map(async d => {
          await store.put(d[0], d[1]);
        }),
      );

      const res0 = await Promise.all(data.map(async d => await store.has(d[0])));
      res0.forEach(res => expect(res).toEqual(true));

      await Promise.all(
        data.map(async d => {
          await store.delete(d[0]);
        }),
      );

      const res1 = await Promise.all(data.map(async d => await store.has(d[0])));
      res1.forEach(res => expect(res).toEqual(false));
    });
  });

  describe('deleteMany', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('streaming', async () => {
      const data = [];
      for (let i = 0; i < 100; i++) {
        data.push({ key: new Key(`/a/key${i}`), value: uint8ArrayFromString(`data${i}`) });
      }

      await drain(store.putMany(data));

      const res0 = await Promise.all(data.map(async d => await store.has(d.key)));
      res0.forEach(res => expect(res).toEqual(true));

      let index = 0;

      for await (const key of store.deleteMany(data.map(d => d.key))) {
        expect(data[index].key).toEqual(key);
        index++;
      }

      expect(index).toEqual(data.length);

      const res1 = await Promise.all(data.map(async d => await store.has(d.key)));
      res1.forEach(res => expect(res).toEqual(false));
    });
  });

  describe('batch', () => {
    let store: D;

    beforeEach(async () => {
      store = await createStore();
    });

    afterEach(async () => {
      await cleanup(store);
    });

    it('simple', async () => {
      const b = store.batch();

      await store.put(new Key('/z/old'), uint8ArrayFromString('old'));

      b.put(new Key('/a/one'), uint8ArrayFromString('1'));
      b.put(new Key('/q/two'), uint8ArrayFromString('2'));
      b.put(new Key('/q/three'), uint8ArrayFromString('3'));
      b.delete(new Key('/z/old'));
      await b.commit();

      const keys = ['/a/one', '/q/two', '/q/three', '/z/old'];
      const res = await Promise.all(keys.map(async k => await store.has(new Key(k))));

      expect(res).toEqual([true, true, true, false]);
    });

    it(
      'many (3 * 400)',
      async function () {
        // this.timeout();
        const b = store.batch();
        const count = 400;
        for (let i = 0; i < count; i++) {
          b.put(new Key(`/a/hello${i}`), randomBytes(32));
          b.put(new Key(`/q/hello${i}`), randomBytes(64));
          b.put(new Key(`/z/hello${i}`), randomBytes(128));
        }

        await b.commit();

        expect(await length(store.query({ prefix: '/a' }))).toEqual(count);
        expect(await length(store.query({ prefix: '/z' }))).toEqual(count);
        expect(await length(store.query({ prefix: '/q' }))).toEqual(count);
      },
      640 * 1000,
    );
  });

  describe('query', () => {
    let store: D;
    const hello = { key: new Key('/q/1hello'), value: uint8ArrayFromString('1') };
    const world = { key: new Key('/z/2world'), value: uint8ArrayFromString('2') };
    const hello2 = { key: new Key('/z/3hello2'), value: uint8ArrayFromString('3') };

    const filter1: QueryFilter = entry => !entry.key.toString().endsWith('hello');
    const filter2: QueryFilter = entry => entry.key.toString().endsWith('hello2');

    const order1: QueryOrder = (a, b) => {
      if (a.value.toString() < b.value.toString()) {
        return -1;
      }
      return 1;
    };
    const order2: QueryOrder = (a, b) => {
      if (a.value.toString() < b.value.toString()) {
        return 1;
      }
      if (a.value.toString() > b.value.toString()) {
        return -1;
      }
      return 0;
    };

    const tests: Array<[string, any, any[] | number]> = [
      ['empty', {}, [hello, world, hello2]],
      ['prefix', { prefix: '/z' }, [world, hello2]],
      ['1 filter', { filters: [filter1] }, [world, hello2]],
      ['2 filters', { filters: [filter1, filter2] }, [hello2]],
      ['limit', { limit: 1 }, 1],
      ['offset', { offset: 1 }, 2],
      ['1 order (1)', { orders: [order1] }, [hello, world, hello2]],
      ['1 order (reverse 1)', { orders: [order2] }, [hello2, world, hello]],
    ];

    beforeAll(async () => {
      store = await createStore();

      const b = store.batch();

      b.put(hello.key, hello.value);
      b.put(world.key, world.value);
      b.put(hello2.key, hello2.value);

      await b.commit();
    });

    afterAll(async () => {
      await cleanup(store);
    });

    tests.forEach(([name, query, expected]) =>
      it(name, async () => {
        let res = await all(store.query(query));

        if (Array.isArray(expected)) {
          if (query.orders == null) {
            expect(res).toHaveLength(expected.length);

            const s: QueryOrder = (a, b) => {
              if (a.key.toString() < b.key.toString()) {
                return 1;
              } else {
                return -1;
              }
            };
            res = res.sort(s);
            const exp = expected.sort(s);

            res.forEach((r, i) => {
              expect(r.key.toString()).toEqual(exp[i].key.toString());

              if (r.value == null) {
                expect(exp[i].value).toBeUndefined();
              } else {
                expect(r.value).toEqual(exp[i].value);
              }
            });
          } else {
            expect(res).toEqual(expected);
          }
        } else if (typeof expected === 'number') {
          expect(res).toHaveLength(expected);
        }
      }),
    );

    it('allows mutating the datastore during a query', async () => {
      const hello3 = { key: new Key('/z/4hello3'), value: uint8ArrayFromString('4') };
      let firstIteration = true;

      // eslint-disable-next-line no-empty-pattern
      for await (const {} of store.query({})) {
        if (firstIteration) {
          expect(await store.has(hello2.key)).toBeTruthy();
          await store.delete(hello2.key);
          expect(await store.has(hello2.key)).toBeFalsy();

          await store.put(hello3.key, hello3.value);
          firstIteration = false;
        }
      }

      const results = await all(store.query({}));

      expect(firstIteration).toBeFalsy(); //('Query did not return anything');
      expect(results.map(result => result.key.toString())).toEqual([
        hello.key.toString(),
        world.key.toString(),
        hello3.key.toString(),
      ]);
    });

    it('queries while the datastore is being mutated', async () => {
      const writePromise = store.put(new Key(`/z/key-${Math.random()}`), uint8ArrayFromString('0'));
      const results = await all(store.query({}));
      expect(results.length).toBeGreaterThan(0);
      await writePromise;
    });
  });

  describe('queryKeys', () => {
    let store: D;
    const hello = { key: new Key('/q/1hello'), value: uint8ArrayFromString('1') };
    const world = { key: new Key('/z/2world'), value: uint8ArrayFromString('2') };
    const hello2 = { key: new Key('/z/3hello2'), value: uint8ArrayFromString('3') };

    const filter1: KeyQueryFilter = key => !key.toString().endsWith('hello');
    const filter2: KeyQueryFilter = key => key.toString().endsWith('hello2');

    const order1: KeyQueryOrder = (a, b) => {
      if (a.toString() < b.toString()) {
        return -1;
      }
      return 1;
    };

    const order2: KeyQueryOrder = (a, b) => {
      if (a.toString() < b.toString()) {
        return 1;
      }
      if (a.toString() > b.toString()) {
        return -1;
      }
      return 0;
    };

    const tests: Array<[string, any, any[] | number]> = [
      ['empty', {}, [hello.key, world.key, hello2.key]],
      ['prefix', { prefix: '/z' }, [world.key, hello2.key]],
      ['1 filter', { filters: [filter1] }, [world.key, hello2.key]],
      ['2 filters', { filters: [filter1, filter2] }, [hello2.key]],
      ['limit', { limit: 1 }, 1],
      ['offset', { offset: 1 }, 2],
      ['1 order (1)', { orders: [order1] }, [hello.key, world.key, hello2.key]],
      ['1 order (reverse 1)', { orders: [order2] }, [hello2.key, world.key, hello.key]],
    ];

    beforeAll(async () => {
      store = await createStore();

      const b = store.batch();

      b.put(hello.key, hello.value);
      b.put(world.key, world.value);
      b.put(hello2.key, hello2.value);

      await b.commit();
    });

    afterAll(async () => {
      await cleanup(store);
    });

    tests.forEach(([name, query, expected]) =>
      it(name, async () => {
        let res = await all(store.queryKeys(query));

        if (Array.isArray(expected)) {
          if (query.orders == null) {
            expect(res).toHaveLength(expected.length);

            const s: KeyQueryOrder = (a, b) => {
              if (a.toString() < b.toString()) {
                return 1;
              } else {
                return -1;
              }
            };
            res = res.sort(s);
            const exp = expected.sort(s);

            res.forEach((r, i) => {
              expect(r.toString()).toEqual(exp[i].toString());
            });
          } else {
            expect(res).toEqual(expected);
          }
        } else if (typeof expected === 'number') {
          expect(res).toHaveLength(expected);
        }
      }),
    );

    it('allows mutating the datastore during a query', async () => {
      const hello3 = { key: new Key('/z/4hello3'), value: uint8ArrayFromString('4') };
      let firstIteration = true;

      // eslint-disable-next-line no-empty-pattern
      for await (const {} of store.queryKeys({})) {
        if (firstIteration) {
          expect(await store.has(hello2.key)).toBeTruthy();
          await store.delete(hello2.key);
          expect(await store.has(hello2.key)).toBeFalsy();

          await store.put(hello3.key, hello3.value);
          firstIteration = false;
        }
      }

      const results = await all(store.queryKeys({}));

      expect(firstIteration).toBeFalsy(); //('Query did not return anything');
      expect(results.map(key => key.toString())).toEqual([
        hello.key.toString(),
        world.key.toString(),
        hello3.key.toString(),
      ]);
    });

    it('queries while the datastore is being mutated', async () => {
      const writePromise = store.put(new Key(`/z/key-${Math.random()}`), uint8ArrayFromString('0'));
      const results = await all(store.queryKeys({}));
      expect(results.length).toBeGreaterThan(0);
      await writePromise;
    });
  });
}
