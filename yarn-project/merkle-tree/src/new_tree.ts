import { type Bufferable, type FromBuffer } from '@aztec/foundation/serialize';
import { type AztecKVStore } from '@aztec/kv-store';
import { type Hasher } from '@aztec/types/interfaces';

import { type TreeBase } from './tree_base.js';

/**
 * Creates a new tree.
 * @param c - The class of the tree to be instantiated.
 * @param db - A database used to store the Merkle tree data.
 * @param hasher - A hasher used to compute hash paths.
 * @param name - Name of the tree.
 * @param depth - Depth of the tree.
 * @param prefilledSize - A number of leaves that are prefilled with values.
 * @returns The newly created tree.
 */
export async function newTree<T extends TreeBase<Bufferable>, D extends FromBuffer<Bufferable>>(
  c: new (store: AztecKVStore, hasher: Hasher, name: string, depth: number, size: bigint, deserializer: D) => T,
  store: AztecKVStore,
  hasher: Hasher,
  name: string,
  deserializer: D,
  depth: number,
  prefilledSize = 1,
): Promise<T> {
  const tree = new c(store, hasher, name, depth, 0n, deserializer);
  await tree.init(prefilledSize);
  return tree;
}
