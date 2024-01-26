import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { randomBytes } from 'crypto';

import { INITIAL_LEAF, Pedersen } from '../index.js';
import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { UpdateOnlyTree } from '../interfaces/update_only_tree.js';
import { appendLeaves } from './utils/append_leaves.js';

const TEST_TREE_DEPTH = 2;

export const standardBasedTreeTestSuite = (
  testName: string,
  createDb: (
    store: AztecKVStore,
    hasher: Hasher,
    name: string,
    depth: number,
  ) => Promise<AppendOnlyTree | UpdateOnlyTree>,
) => {
  describe(testName, () => {
    let pedersen: Pedersen;
    const values: Buffer[] = [];

    beforeAll(() => {
      pedersen = new Pedersen();

      for (let i = 0; i < 4; ++i) {
        const v = Buffer.alloc(32, i + 1);
        v.writeUInt32BE(i, 28);
        values[i] = v;
      }
    });

    it('should have correct empty tree root for depth 32', async () => {
      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test', 32);
      const root = tree.getRoot(false);
      expect(root.toString('hex')).toEqual('16642d9ccd8346c403aa4c3fa451178b22534a27035cdaa6ec34ae53b29c50cb');
    });

    it('should throw when appending beyond max index', async () => {
      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test', 2);
      const leaves = Array.from({ length: 5 }, _ => randomBytes(32));
      await expect(appendLeaves(tree, leaves)).rejects.toThrow();
    });

    it('should have correct root and sibling paths', async () => {
      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test', 2);

      const level1ZeroHash = pedersen.hash(INITIAL_LEAF, INITIAL_LEAF);
      expect(tree.getNumLeaves(false)).toEqual(0n);
      expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));
      expect(await tree.getSiblingPath(0n, false)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );

      await appendLeaves(tree, [values[0]]);
      expect(tree.getNumLeaves(true)).toEqual(1n);
      expect(tree.getNumLeaves(false)).toEqual(0n);
      expect(tree.getRoot(true)).toEqual(pedersen.hash(pedersen.hash(values[0], INITIAL_LEAF), level1ZeroHash));
      expect(await tree.getSiblingPath(0n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );
      expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));
      expect(await tree.getSiblingPath(0n, false)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );

      await appendLeaves(tree, [values[1]]);
      expect(tree.getNumLeaves(true)).toEqual(2n);
      expect(tree.getRoot(true)).toEqual(pedersen.hash(pedersen.hash(values[0], values[1]), level1ZeroHash));
      expect(await tree.getSiblingPath(1n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[0], level1ZeroHash]),
      );
      expect(tree.getNumLeaves(false)).toEqual(0n);
      expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));
      expect(await tree.getSiblingPath(1n, false)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );

      await appendLeaves(tree, [values[2]]);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      expect(tree.getRoot(true)).toEqual(
        pedersen.hash(pedersen.hash(values[0], values[1]), pedersen.hash(values[2], INITIAL_LEAF)),
      );
      expect(await tree.getSiblingPath(2n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, pedersen.hash(values[0], values[1])]),
      );
      expect(tree.getNumLeaves(false)).toEqual(0n);
      expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));
      expect(await tree.getSiblingPath(2n, false)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );

      await appendLeaves(tree, [values[3]]);
      expect(tree.getNumLeaves(true)).toEqual(4n);
      expect(tree.getRoot(true)).toEqual(
        pedersen.hash(pedersen.hash(values[0], values[1]), pedersen.hash(values[2], values[3])),
      );
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[2], pedersen.hash(values[0], values[1])]),
      );
      expect(tree.getNumLeaves(false)).toEqual(0n);
      expect(tree.getRoot(false)).toEqual(pedersen.hash(level1ZeroHash, level1ZeroHash));
      expect(await tree.getSiblingPath(3n, false)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash]),
      );
      // Lifted from memory_tree.test.cpp to ensure consistency.
      //expect(root.toString('hex')).toEqual('0bf2e78afd70f72b0e6eafb03c41faef167a82441b05e517cdf35d813302061f');
      expect(await tree.getSiblingPath(0n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[1], pedersen.hash(values[2], values[3])]),
      );
      expect(await tree.getSiblingPath(1n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[0], pedersen.hash(values[2], values[3])]),
      );
      expect(await tree.getSiblingPath(2n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[3], pedersen.hash(values[0], values[1])]),
      );
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [values[2], pedersen.hash(values[0], values[1])]),
      );

      await tree.commit();
      // now committed state should equal uncommitted state
      expect(await tree.getSiblingPath(0n, false)).toEqual(await tree.getSiblingPath(0n, true));
      expect(await tree.getSiblingPath(1n, false)).toEqual(await tree.getSiblingPath(1n, true));
      expect(await tree.getSiblingPath(2n, false)).toEqual(await tree.getSiblingPath(2n, true));
      expect(await tree.getSiblingPath(3n, false)).toEqual(await tree.getSiblingPath(3n, true));
      expect(tree.getNumLeaves(false)).toEqual(tree.getNumLeaves(true));
      expect(tree.getRoot(false)).toEqual(tree.getRoot(true));
    });
  });
};
