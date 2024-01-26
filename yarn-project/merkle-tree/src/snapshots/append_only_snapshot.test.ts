import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';

import { Pedersen, StandardTree, newTree } from '../index.js';
import { AppendOnlySnapshotBuilder } from './append_only_snapshot.js';
import { describeSnapshotBuilderTestSuite } from './snapshot_builder_test_suite.js';

describe('AppendOnlySnapshot', () => {
  let tree: StandardTree;
  let snapshotBuilder: AppendOnlySnapshotBuilder;
  let db: AztecKVStore;

  beforeEach(async () => {
    db = await AztecLmdbStore.openTmp();
    const hasher = new Pedersen();
    tree = await newTree(StandardTree, db, hasher, 'test', 4);
    snapshotBuilder = new AppendOnlySnapshotBuilder(db, tree, hasher);
  });

  describeSnapshotBuilderTestSuite(
    () => tree,
    () => snapshotBuilder,
    async tree => {
      const newLeaves = Array.from({ length: 2 }).map(() => Buffer.from(Math.random().toString()));
      await tree.appendLeaves(newLeaves);
    },
  );
});
