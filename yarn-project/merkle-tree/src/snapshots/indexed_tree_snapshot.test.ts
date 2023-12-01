import levelup, { LevelUp } from 'levelup';

import { Pedersen, newTree } from '../index.js';
import { StandardIndexedTreeWithAppend } from '../standard_indexed_tree/test/standard_indexed_tree_with_append.js';
import { createMemDown } from '../test/utils/create_mem_down.js';
import { IndexedTreeSnapshotBuilder } from './indexed_tree_snapshot.js';
import { describeSnapshotBuilderTestSuite } from './snapshot_builder_test_suite.js';

describe('IndexedTreeSnapshotBuilder', () => {
  let db: LevelUp;
  let tree: StandardIndexedTreeWithAppend;
  let snapshotBuilder: IndexedTreeSnapshotBuilder;

  beforeEach(async () => {
    db = levelup(createMemDown());
    tree = await newTree(StandardIndexedTreeWithAppend, db, new Pedersen(), 'test', 4);
    snapshotBuilder = new IndexedTreeSnapshotBuilder(db, tree);
  });

  describeSnapshotBuilderTestSuite(
    () => tree,
    () => snapshotBuilder,
    async () => {
      const newLeaves = Array.from({ length: 2 }).map(() => Buffer.from(Math.random().toString()));
      await tree.appendLeaves(newLeaves);
    },
  );

  describe('getSnapshot', () => {
    it('returns historical leaf data', async () => {
      await tree.appendLeaves([Buffer.from('a'), Buffer.from('b'), Buffer.from('c')]);
      await tree.commit();
      const expectedLeavesAtBlock1 = await Promise.all([
        tree.getLatestLeafDataCopy(0, false),
        tree.getLatestLeafDataCopy(1, false),
        tree.getLatestLeafDataCopy(2, false),
        // id'expect these to be undefined, but leaf 3 isn't?
        // must be some indexed-tree quirk I don't quite understand yet
        tree.getLatestLeafDataCopy(3, false),
        tree.getLatestLeafDataCopy(4, false),
        tree.getLatestLeafDataCopy(5, false),
      ]);

      await snapshotBuilder.snapshot(1);

      await tree.appendLeaves([Buffer.from('d'), Buffer.from('e'), Buffer.from('f')]);
      await tree.commit();
      const expectedLeavesAtBlock2 = await Promise.all([
        tree.getLatestLeafDataCopy(0, false),
        tree.getLatestLeafDataCopy(1, false),
        tree.getLatestLeafDataCopy(2, false),
        tree.getLatestLeafDataCopy(3, false),
        tree.getLatestLeafDataCopy(4, false),
        tree.getLatestLeafDataCopy(5, false),
      ]);

      await snapshotBuilder.snapshot(2);

      const snapshot1 = await snapshotBuilder.getSnapshot(1);
      const actualLeavesAtBlock1 = await Promise.all([
        snapshot1.getLatestLeafDataCopy(0n),
        snapshot1.getLatestLeafDataCopy(1n),
        snapshot1.getLatestLeafDataCopy(2n),
        snapshot1.getLatestLeafDataCopy(3n),
        snapshot1.getLatestLeafDataCopy(4n),
        snapshot1.getLatestLeafDataCopy(5n),
      ]);
      expect(actualLeavesAtBlock1).toEqual(expectedLeavesAtBlock1);

      const snapshot2 = await snapshotBuilder.getSnapshot(2);
      const actualLeavesAtBlock2 = await Promise.all([
        snapshot2.getLatestLeafDataCopy(0n),
        snapshot2.getLatestLeafDataCopy(1n),
        snapshot2.getLatestLeafDataCopy(2n),
        snapshot2.getLatestLeafDataCopy(3n),
        snapshot2.getLatestLeafDataCopy(4n),
        snapshot2.getLatestLeafDataCopy(5n),
      ]);
      expect(actualLeavesAtBlock2).toEqual(expectedLeavesAtBlock2);
    });
  });

  describe('findIndexOfPreviousValue', () => {
    it('returns the index of the leaf with the closest value to the given value', async () => {
      await tree.appendLeaves([Buffer.from('a'), Buffer.from('f'), Buffer.from('d')]);
      await tree.commit();
      const snapshot = await snapshotBuilder.snapshot(1);
      const historicalPrevValue = tree.findIndexOfPreviousValue(2n, false);

      await tree.appendLeaves([Buffer.from('c'), Buffer.from('b'), Buffer.from('e')]);
      await tree.commit();

      await expect(snapshot.findIndexOfPreviousValue(2n)).resolves.toEqual(historicalPrevValue);
    });
  });
});
