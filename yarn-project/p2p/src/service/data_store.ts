import { filter, map, sort, take } from '@aztec/foundation/iterable';
import type { AztecKVStore, AztecMap } from '@aztec/kv-store';

import { type Batch, type Datastore, Key, type KeyQuery, type Pair, type Query } from 'interface-datastore';
import type { AwaitIterable } from 'interface-store';

type MemoryItem = {
  lastAccessedMs: number;
  data: Uint8Array;
};

type BatchOp = {
  type: 'put' | 'del';
  key: Key;
  value?: Uint8Array;
};

class KeyNotFoundError extends Error {
  code: string;
  constructor(message: string) {
    super(message);
    this.code = 'ERR_NOT_FOUND';
  }
}

export class AztecDatastore implements Datastore {
  #memoryDatastore: Map<string, MemoryItem>;
  #dbDatastore: AztecMap<string, Uint8Array>;

  #batchOps: BatchOp[] = [];

  private maxMemoryItems: number;

  constructor(db: AztecKVStore, { maxMemoryItems } = { maxMemoryItems: 50 }) {
    this.#memoryDatastore = new Map();
    this.#dbDatastore = db.openMap('p2p_datastore');

    this.maxMemoryItems = maxMemoryItems;
  }

  has(key: Key): boolean {
    return this.#memoryDatastore.has(key.toString()) || this.#dbDatastore.has(key.toString());
  }

  get(key: Key): Uint8Array {
    const keyStr = key.toString();
    const memoryItem = this.#memoryDatastore.get(keyStr);
    if (memoryItem) {
      memoryItem.lastAccessedMs = Date.now();
      return memoryItem.data;
    }
    const dbItem = this.#dbDatastore.get(keyStr);

    if (!dbItem) {
      throw new KeyNotFoundError(`Key not found`);
    }

    return Uint8Array.from(dbItem);
  }

  put(key: Key, val: Uint8Array): Promise<Key> {
    return this._put(key, val);
  }

  async *putMany(source: AwaitIterable<Pair>): AwaitIterable<Key> {
    for await (const { key, value } of source) {
      await this.put(key, value);
      yield key;
    }
  }

  async *getMany(source: AwaitIterable<Key>): AwaitIterable<Pair> {
    for await (const key of source) {
      yield {
        key,
        value: this.get(key),
      };
    }
  }

  async *deleteMany(source: AwaitIterable<Key>): AwaitIterable<Key> {
    for await (const key of source) {
      await this.delete(key);
      yield key;
    }
  }

  async delete(key: Key): Promise<void> {
    this.#memoryDatastore.delete(key.toString());
    await this.#dbDatastore.delete(key.toString());
  }

  batch(): Batch {
    return {
      put: (key, value) => {
        this.#batchOps.push({
          type: 'put',
          key,
          value,
        });
      },
      delete: key => {
        this.#batchOps.push({
          type: 'del',
          key,
        });
      },
      commit: async () => {
        for (const op of this.#batchOps) {
          if (op.type === 'put' && op.value) {
            await this.put(op.key, op.value);
          } else if (op.type === 'del') {
            await this.delete(op.key);
          }
        }
        this.#batchOps = []; // Clear operations after commit
      },
    };
  }

  query(q: Query): AwaitIterable<Pair> {
    let it = this.all(); //
    const { prefix, filters, orders, offset, limit } = q;

    if (prefix != null) {
      it = filter(it, e => e.key.toString().startsWith(`${prefix}`));
    }

    if (Array.isArray(filters)) {
      it = filters.reduce((it, f) => filter(it, f), it);
    }

    if (Array.isArray(orders)) {
      it = orders.reduce((it, f) => sort(it, f), it);
    }

    if (offset != null) {
      let i = 0;
      it = filter(it, () => i++ >= offset);
    }

    if (limit != null) {
      it = take(it, limit);
    }

    return it;
  }

  queryKeys(q: KeyQuery): AsyncIterable<Key> {
    let it = map(this.all(), ({ key }) => key);
    const { prefix, filters, orders, offset, limit } = q;
    if (prefix != null) {
      it = filter(it, e => e.toString().startsWith(`${prefix}`));
    }

    if (Array.isArray(filters)) {
      it = filters.reduce((it, f) => filter(it, f), it);
    }

    if (Array.isArray(orders)) {
      it = orders.reduce((it, f) => sort(it, f), it);
    }

    if (offset != null) {
      let i = 0;
      it = filter(it, () => i++ >= offset);
    }

    if (limit != null) {
      it = take(it, limit);
    }

    return it;
  }

  private async _put(key: Key, val: Uint8Array): Promise<Key> {
    const keyStr = key.toString();
    while (this.#memoryDatastore.size >= this.maxMemoryItems) {
      this.pruneMemoryDatastore();
    }
    const memoryItem = this.#memoryDatastore.get(keyStr);
    if (memoryItem) {
      // update existing
      memoryItem.lastAccessedMs = Date.now();
      memoryItem.data = val;
    } else {
      // new entry
      this.#memoryDatastore.set(keyStr, { data: val, lastAccessedMs: Date.now() });
    }

    // Always add to DB
    await this.#dbDatastore.set(keyStr, val);

    return key;
  }

  private async *all(): AsyncIterable<Pair> {
    for (const [key, value] of this.#memoryDatastore.entries()) {
      yield {
        key: new Key(key),
        value: value.data,
      };
    }

    for (const [key, value] of this.#dbDatastore.entries()) {
      if (!this.#memoryDatastore.has(key)) {
        yield {
          key: new Key(key),
          value,
        };
      }
    }
  }

  /**
   * Prune memory store
   */
  private pruneMemoryDatastore(): void {
    let oldestAccessedMs = Date.now() + 1000;
    let oldestKey: string | undefined = undefined;
    let oldestValue: Uint8Array | undefined = undefined;

    for (const [key, value] of this.#memoryDatastore) {
      if (value.lastAccessedMs < oldestAccessedMs) {
        oldestAccessedMs = value.lastAccessedMs;
        oldestKey = key;
        oldestValue = value.data;
      }
    }

    if (oldestKey && oldestValue) {
      this.#memoryDatastore.delete(oldestKey);
    }
  }
}
