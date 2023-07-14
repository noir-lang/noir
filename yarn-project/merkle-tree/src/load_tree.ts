import { Hasher } from '@aztec/types';

import { LevelUp } from 'levelup';

import { TreeBase, decodeMeta } from './tree_base.js';

/**
 * Creates a new tree and sets its root, depth and size based on the meta data which are associated with the name.
 * @param c - The class of the tree to be instantiated.
 * @param db - A database used to store the Merkle tree data.
 * @param hasher - A hasher used to compute hash paths.
 * @param name - Name of the tree.
 * @returns The newly created tree.
 */
export async function loadTree<T extends TreeBase>(
  c: new (...args: any[]) => T,
  db: LevelUp,
  hasher: Hasher,
  name: string,
): Promise<T> {
  const meta: Buffer = await db.get(name);
  const { root, depth, size } = decodeMeta(meta);
  const tree = new c(db, hasher, name, depth, size, root);
  await tree.initFromDb();
  return tree;
}
