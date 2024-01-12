import { Hasher } from '@aztec/types/interfaces';

import { LevelUp } from 'levelup';

import { TreeBase } from './tree_base.js';

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
export async function newTree<T extends TreeBase>(
  c: new (db: LevelUp, hasher: Hasher, name: string, depth: number, size: bigint) => T,
  db: LevelUp,
  hasher: Hasher,
  name: string,
  depth: number,
  prefilledSize = 1,
): Promise<T> {
  const tree = new c(db, hasher, name, depth, 0n);
  await tree.init(prefilledSize);
  return tree;
}
