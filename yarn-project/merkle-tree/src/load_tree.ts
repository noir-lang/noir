import { AztecKVStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';

import { TreeBase, getTreeMeta } from './tree_base.js';

/**
 * Creates a new tree and sets its root, depth and size based on the meta data which are associated with the name.
 * @param c - The class of the tree to be instantiated.
 * @param db - A database used to store the Merkle tree data.
 * @param hasher - A hasher used to compute hash paths.
 * @param name - Name of the tree.
 * @returns The newly created tree.
 */
export function loadTree<T extends TreeBase>(
  c: new (store: AztecKVStore, hasher: Hasher, name: string, depth: number, size: bigint, root: Buffer) => T,
  store: AztecKVStore,
  hasher: Hasher,
  name: string,
): Promise<T> {
  const { root, depth, size } = getTreeMeta(store, name);
  const tree = new c(store, hasher, name, depth, size, root);
  return Promise.resolve(tree);
}
