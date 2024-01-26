import { Fr, NullifierLeaf, NullifierLeafPreimage } from '@aztec/circuits.js';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';

import { Pedersen, newTree } from '../index.js';
import { StandardIndexedTreeWithAppend } from '../standard_indexed_tree/test/standard_indexed_tree_with_append.js';
import { IndexedTreeSnapshotBuilder } from './indexed_tree_snapshot.js';
import { describeSnapshotBuilderTestSuite } from './snapshot_builder_test_suite.js';

class NullifierTree extends StandardIndexedTreeWithAppend {
  constructor(db: AztecKVStore, hasher: Hasher, name: string, depth: number, size: bigint = 0n, root?: Buffer) {
    super(db, hasher, name, depth, size, NullifierLeafPreimage, NullifierLeaf, root);
  }
}

describe('IndexedTreeSnapshotBuilder', () => {
  let db: AztecKVStore;
  let tree: StandardIndexedTreeWithAppend;
  let snapshotBuilder: IndexedTreeSnapshotBuilder;

  beforeEach(async () => {
    db = await AztecLmdbStore.openTmp();
    tree = await newTree(NullifierTree, db, new Pedersen(), 'test', 4);
    snapshotBuilder = new IndexedTreeSnapshotBuilder(db, tree, NullifierLeafPreimage);
  });

  describeSnapshotBuilderTestSuite(
    () => tree,
    () => snapshotBuilder,
    async () => {
      const newLeaves = Array.from({ length: 2 }).map(() => new NullifierLeaf(Fr.random()).toBuffer());
      await tree.appendLeaves(newLeaves);
    },
  );

  describe('getSnapshot', () => {
    it('returns historical leaf data', async () => {
      await tree.appendLeaves([Buffer.from('a'), Buffer.from('b'), Buffer.from('c')]);
      await tree.commit();
      const expectedLeavesAtBlock1 = await Promise.all([
        tree.getLatestLeafPreimageCopy(0n, false),
        tree.getLatestLeafPreimageCopy(1n, false),
        tree.getLatestLeafPreimageCopy(2n, false),
        // id'expect these to be undefined, but leaf 3 isn't?
        // must be some indexed-tree quirk I don't quite understand yet
        tree.getLatestLeafPreimageCopy(3n, false),
        tree.getLatestLeafPreimageCopy(4n, false),
        tree.getLatestLeafPreimageCopy(5n, false),
      ]);

      await snapshotBuilder.snapshot(1);

      await tree.appendLeaves([Buffer.from('d'), Buffer.from('e'), Buffer.from('f')]);
      await tree.commit();
      const expectedLeavesAtBlock2 = [
        tree.getLatestLeafPreimageCopy(0n, false),
        tree.getLatestLeafPreimageCopy(1n, false),
        tree.getLatestLeafPreimageCopy(2n, false),
        tree.getLatestLeafPreimageCopy(3n, false),
        tree.getLatestLeafPreimageCopy(4n, false),
        tree.getLatestLeafPreimageCopy(5n, false),
      ];

      await snapshotBuilder.snapshot(2);

      const snapshot1 = await snapshotBuilder.getSnapshot(1);
      const actualLeavesAtBlock1 = [
        snapshot1.getLatestLeafPreimageCopy(0n),
        snapshot1.getLatestLeafPreimageCopy(1n),
        snapshot1.getLatestLeafPreimageCopy(2n),
        snapshot1.getLatestLeafPreimageCopy(3n),
        snapshot1.getLatestLeafPreimageCopy(4n),
        snapshot1.getLatestLeafPreimageCopy(5n),
      ];
      expect(actualLeavesAtBlock1).toEqual(expectedLeavesAtBlock1);

      const snapshot2 = await snapshotBuilder.getSnapshot(2);
      const actualLeavesAtBlock2 = await Promise.all([
        snapshot2.getLatestLeafPreimageCopy(0n),
        snapshot2.getLatestLeafPreimageCopy(1n),
        snapshot2.getLatestLeafPreimageCopy(2n),
        snapshot2.getLatestLeafPreimageCopy(3n),
        snapshot2.getLatestLeafPreimageCopy(4n),
        snapshot2.getLatestLeafPreimageCopy(5n),
      ]);
      expect(actualLeavesAtBlock2).toEqual(expectedLeavesAtBlock2);
    });
  });

  describe('findIndexOfPreviousValue', () => {
    it('returns the index of the leaf with the closest value to the given value', async () => {
      await tree.appendLeaves([Buffer.from('a'), Buffer.from('f'), Buffer.from('d')]);
      await tree.commit();
      const snapshot = await snapshotBuilder.snapshot(1);
      const historicalPrevValue = tree.findIndexOfPreviousKey(2n, false);

      await tree.appendLeaves([Buffer.from('c'), Buffer.from('b'), Buffer.from('e')]);
      await tree.commit();

      expect(snapshot.findIndexOfPreviousKey(2n)).toEqual(historicalPrevValue);
    });
  });
});
