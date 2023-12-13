import { Database, Key } from 'lmdb';

import { AztecMultiMap } from '../interfaces/map.js';

/** The slot where a key-value entry would be stored */
type MapKeyValueSlot<K extends string | number> = ['map', string, 'slot', K];

/**
 * A map backed by LMDB.
 */
export class LmdbAztecMap<K extends string | number, V> implements AztecMultiMap<K, V> {
  protected db: Database<V, MapKeyValueSlot<K>>;
  protected name: string;

  constructor(rootDb: Database<unknown, Key>, mapName: string) {
    this.name = mapName;
    this.db = rootDb as Database<V, MapKeyValueSlot<K>>;
  }

  close(): Promise<void> {
    return this.db.close();
  }

  get(key: K): V | undefined {
    return this.db.get(this.#slot(key)) as V | undefined;
  }

  *getValues(key: K): IterableIterator<V> {
    const values = this.db.getValues(this.#slot(key));
    for (const value of values) {
      yield value;
    }
  }

  has(key: K): boolean {
    return this.db.doesExist(this.#slot(key));
  }

  set(key: K, val: V): Promise<boolean> {
    return this.db.put(this.#slot(key), val);
  }

  setIfNotExists(key: K, val: V): Promise<boolean> {
    const slot = this.#slot(key);
    return this.db.ifNoExists(slot, () => {
      void this.db.put(slot, val);
    });
  }

  delete(key: K): Promise<boolean> {
    return this.db.remove(this.#slot(key));
  }

  async deleteValue(key: K, val: V): Promise<void> {
    await this.db.remove(this.#slot(key), val);
  }

  *entries(): IterableIterator<[K, V]> {
    const iterator = this.db.getRange({
      start: ['map', this.name, 'slot'],
    });

    for (const { key, value } of iterator) {
      if (key[0] !== 'map' || key[1] !== this.name) {
        break;
      }

      const originalKey = key[3];
      yield [originalKey, value];
    }
  }

  *values(): IterableIterator<V> {
    for (const [_, value] of this.entries()) {
      yield value;
    }
  }

  *keys(): IterableIterator<K> {
    for (const [key, _] of this.entries()) {
      yield key;
    }
  }

  #slot(key: K): MapKeyValueSlot<K> {
    return ['map', this.name, 'slot', key];
  }
}
