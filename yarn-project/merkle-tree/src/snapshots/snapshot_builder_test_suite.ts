import { TreeBase } from '../tree_base.js';
import { TreeSnapshotBuilder } from './snapshot_builder.js';

/** Creates a test suit for snapshots */
export function describeSnapshotBuilderTestSuite<T extends TreeBase, S extends TreeSnapshotBuilder>(
  getTree: () => T,
  getSnapshotBuilder: () => S,
  modifyTree: (tree: T) => Promise<void>,
) {
  describe('SnapshotBuilder', () => {
    let tree: T;
    let snapshotBuilder: S;
    let leaves: bigint[];

    beforeEach(() => {
      tree = getTree();
      snapshotBuilder = getSnapshotBuilder();

      leaves = Array.from({ length: 4 }).map(() => BigInt(Math.floor(Math.random() * 2 ** tree.getDepth())));
    });

    describe('snapshot', () => {
      it('takes snapshots', async () => {
        await modifyTree(tree);
        await tree.commit();
        await expect(snapshotBuilder.snapshot(1)).resolves.toBeDefined();
      });

      it('is idempotent', async () => {
        await modifyTree(tree);
        await tree.commit();

        const block = 1;
        const snapshot = await snapshotBuilder.snapshot(block);
        await expect(snapshotBuilder.snapshot(block)).resolves.toEqual(snapshot);
      });

      it('returns the same path if tree has not diverged', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);

        const historicPaths = await Promise.all(leaves.map(leaf => snapshot.getSiblingPath(leaf)));
        const expectedPaths = await Promise.all(leaves.map(leaf => tree.getSiblingPath(leaf, false)));

        for (const [index, path] of historicPaths.entries()) {
          expect(path).toEqual(expectedPaths[index]);
        }
      });

      it('returns historic paths if tree has diverged and no new snapshots have been taken', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);

        const expectedPaths = await Promise.all(leaves.map(leaf => tree.getSiblingPath(leaf, false)));

        await modifyTree(tree);
        await tree.commit();

        const historicPaths = await Promise.all(leaves.map(leaf => snapshot.getSiblingPath(leaf)));

        for (const [index, path] of historicPaths.entries()) {
          expect(path).toEqual(expectedPaths[index]);
        }
      });

      it('retains old snapshots even if new one are created', async () => {
        await modifyTree(tree);
        await tree.commit();

        const expectedPaths = await Promise.all(leaves.map(leaf => tree.getSiblingPath(leaf, false)));

        const snapshot = await snapshotBuilder.snapshot(1);

        await modifyTree(tree);
        await tree.commit();

        await snapshotBuilder.snapshot(2);

        // check that snapshot 2 has not influenced snapshot(1) at all
        const historicPaths = await Promise.all(leaves.map(leaf => snapshot.getSiblingPath(leaf)));

        for (const [index, path] of historicPaths.entries()) {
          expect(path).toEqual(expectedPaths[index]);
        }
      });

      it('retains old snapshots even if new one are created and the tree diverges', async () => {
        await modifyTree(tree);
        await tree.commit();

        const expectedPaths = await Promise.all(leaves.map(leaf => tree.getSiblingPath(leaf, false)));

        const snapshot = await snapshotBuilder.snapshot(1);

        await modifyTree(tree);
        await tree.commit();

        await snapshotBuilder.snapshot(2);

        await modifyTree(tree);
        await tree.commit();

        // check that snapshot 2 has not influenced snapshot(1) at all
        // and that the diverging tree does not influence the old snapshot
        const historicPaths = await Promise.all(leaves.map(leaf => snapshot.getSiblingPath(leaf)));

        for (const [index, path] of historicPaths.entries()) {
          expect(path).toEqual(expectedPaths[index]);
        }
      });
    });

    describe('getSnapshot', () => {
      it('returns old snapshots', async () => {
        await modifyTree(tree);
        await tree.commit();
        const expectedPaths = await Promise.all(leaves.map(leaf => tree.getSiblingPath(leaf, false)));
        await snapshotBuilder.snapshot(1);

        for (let i = 2; i < 5; i++) {
          await modifyTree(tree);
          await tree.commit();
          await snapshotBuilder.snapshot(i);
        }

        const firstSnapshot = await snapshotBuilder.getSnapshot(1);
        const historicPaths = await Promise.all(leaves.map(leaf => firstSnapshot.getSiblingPath(leaf)));

        for (const [index, path] of historicPaths.entries()) {
          expect(path).toEqual(expectedPaths[index]);
        }
      });

      it('throws if an unknown snapshot is requested', async () => {
        await modifyTree(tree);
        await tree.commit();
        await snapshotBuilder.snapshot(1);

        await expect(snapshotBuilder.getSnapshot(2)).rejects.toThrow();
      });
    });

    describe('getRoot', () => {
      it('returns the historical root of the tree when the snapshot was taken', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);
        const historicalRoot = tree.getRoot(false);

        await modifyTree(tree);
        await tree.commit();

        expect(snapshot.getRoot()).toEqual(historicalRoot);
        expect(snapshot.getRoot()).not.toEqual(tree.getRoot(false));
      });
    });

    describe('getDepth', () => {
      it('returns the same depth as the tree', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);
        expect(snapshot.getDepth()).toEqual(tree.getDepth());
      });
    });

    describe('getNumLeaves', () => {
      it('returns the historical leaves count when the snapshot was taken', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);
        const historicalNumLeaves = tree.getNumLeaves(false);

        await modifyTree(tree);
        await tree.commit();

        expect(snapshot.getNumLeaves()).toEqual(historicalNumLeaves);
      });
    });

    describe('getLeafValue', () => {
      it('returns the historical leaf value when the snapshot was taken', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);
        const historicalLeafValue = tree.getLeafValue(0n, false);
        expect(snapshot.getLeafValue(0n)).toEqual(historicalLeafValue);

        await modifyTree(tree);
        await tree.commit();

        expect(snapshot.getLeafValue(0n)).toEqual(historicalLeafValue);
      });
    });

    describe('findLeafIndex', () => {
      it('returns the historical leaf index when the snapshot was taken', async () => {
        await modifyTree(tree);
        await tree.commit();
        const snapshot = await snapshotBuilder.snapshot(1);

        const initialLastLeafIndex = tree.getNumLeaves(false) - 1n;
        let lastLeaf = tree.getLeafValue(initialLastLeafIndex, false);
        expect(snapshot.findLeafIndex(lastLeaf!)).toBe(initialLastLeafIndex);

        await modifyTree(tree);
        await tree.commit();

        const newLastLeafIndex = tree.getNumLeaves(false) - 1n;
        lastLeaf = tree.getLeafValue(newLastLeafIndex, false);

        expect(snapshot.findLeafIndex(lastLeaf!)).toBe(undefined);
      });
    });
  });
}
