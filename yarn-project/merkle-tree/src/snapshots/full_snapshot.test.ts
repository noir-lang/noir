import { randomBytes } from '@aztec/foundation/crypto';
import { AztecKVStore } from '@aztec/kv-store';
import { openTmpStore } from '@aztec/kv-store/utils';

import { Pedersen, StandardTree, newTree } from '../index.js';
import { FullTreeSnapshotBuilder } from './full_snapshot.js';
import { describeSnapshotBuilderTestSuite } from './snapshot_builder_test_suite.js';

describe('FullSnapshotBuilder', () => {
  let tree: StandardTree;
  let snapshotBuilder: FullTreeSnapshotBuilder;
  let db: AztecKVStore;

  beforeEach(async () => {
    db = openTmpStore();
    tree = await newTree(StandardTree, db, new Pedersen(), 'test', 4);
    snapshotBuilder = new FullTreeSnapshotBuilder(db, tree);
  });

  describeSnapshotBuilderTestSuite(
    () => tree,
    () => snapshotBuilder,
    async () => {
      const newLeaves = Array.from({ length: 2 }).map(() => randomBytes(32));
      await tree.appendLeaves(newLeaves);
    },
  );
});
