import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { Pedersen } from '../index.js';
import { AppendOnlyTree } from '../interfaces/append_only_tree.js';
import { UpdateOnlyTree } from '../interfaces/update_only_tree.js';
import { appendLeaves } from './utils/append_leaves.js';

const expectSameTrees = async (
  tree1: AppendOnlyTree | UpdateOnlyTree,
  tree2: AppendOnlyTree | UpdateOnlyTree,
  includeUncommitted = true,
) => {
  const size = tree1.getNumLeaves(includeUncommitted);
  expect(size).toBe(tree2.getNumLeaves(includeUncommitted));
  expect(tree1.getRoot(includeUncommitted).toString('hex')).toBe(tree2.getRoot(includeUncommitted).toString('hex'));

  for (let i = 0; i < size; ++i) {
    const siblingPath1 = await tree1.getSiblingPath(BigInt(i), includeUncommitted);
    const siblingPath2 = await tree2.getSiblingPath(BigInt(i), includeUncommitted);
    expect(siblingPath2).toStrictEqual(siblingPath1);
  }
};

export const treeTestSuite = (
  testName: string,
  createDb: (
    store: AztecKVStore,
    hasher: Hasher,
    name: string,
    depth: number,
  ) => Promise<AppendOnlyTree | UpdateOnlyTree>,
  createFromName: (store: AztecKVStore, hasher: Hasher, name: string) => Promise<AppendOnlyTree | UpdateOnlyTree>,
) => {
  describe(testName, () => {
    const values: Buffer[] = [];
    let pedersen: Pedersen;

    beforeAll(() => {
      for (let i = 0; i < 32; ++i) {
        const v = Buffer.alloc(32, i + 1);
        v.writeUInt32BE(i, 28);
        values[i] = v;
      }
    });

    beforeEach(() => {
      pedersen = new Pedersen();
    });

    it('should revert changes on rollback', async () => {
      const dbEmpty = await AztecLmdbStore.openTmp();
      const emptyTree = await createDb(dbEmpty, pedersen, 'test', 10);

      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test2', 10);
      await appendLeaves(tree, values.slice(0, 4));

      const firstRoot = tree.getRoot(true);
      expect(firstRoot).not.toEqual(emptyTree.getRoot(true));
      // committed root should still be the empty root
      expect(tree.getRoot(false)).toEqual(emptyTree.getRoot(false));

      await tree.rollback();

      // both committed and uncommitted trees should be equal to the empty tree
      await expectSameTrees(tree, emptyTree, true);
      await expectSameTrees(tree, emptyTree, false);

      // append the leaves again
      await appendLeaves(tree, values.slice(0, 4));

      expect(tree.getRoot(true)).toEqual(firstRoot);
      // committed root should still be the empty root
      expect(tree.getRoot(false)).toEqual(emptyTree.getRoot(false));

      expect(firstRoot).not.toEqual(emptyTree.getRoot(true));

      await tree.rollback();

      // both committed and uncommitted trees should be equal to the empty tree
      await expectSameTrees(tree, emptyTree, true);
      await expectSameTrees(tree, emptyTree, false);
    });

    it('should not revert changes after commit', async () => {
      const dbEmpty = await AztecLmdbStore.openTmp();
      const emptyTree = await createDb(dbEmpty, pedersen, 'test', 10);

      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test2', 10);
      await appendLeaves(tree, values.slice(0, 4));

      expect(tree.getRoot(true)).not.toEqual(emptyTree.getRoot(true));
      // committed root should still be the empty root
      expect(tree.getRoot(false)).toEqual(emptyTree.getRoot(false));

      await tree.commit();
      await tree.rollback();

      expect(tree.getRoot(true)).not.toEqual(emptyTree.getRoot(true));
      expect(tree.getRoot(false)).not.toEqual(emptyTree.getRoot(true));
    });

    it('should be able to restore from previous committed data', async () => {
      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test', 10);
      await appendLeaves(tree, values.slice(0, 4));
      await tree.commit();

      const tree2 = await createFromName(db, pedersen, 'test');

      // both committed and uncommitted should be equal to the restored data
      expect(tree.getRoot(true)).toEqual(tree2.getRoot(true));
      expect(tree.getRoot(false)).toEqual(tree2.getRoot(false));
      for (let i = 0; i < 4; ++i) {
        expect(await tree.getSiblingPath(BigInt(i), true)).toEqual(await tree2.getSiblingPath(BigInt(i), true));
        expect(await tree.getSiblingPath(BigInt(i), false)).toEqual(await tree2.getSiblingPath(BigInt(i), false));
      }
    });

    it('should throw an error if previous data does not exist for the given name', async () => {
      const db = await AztecLmdbStore.openTmp();
      await expect(
        (async () => {
          await createFromName(db, pedersen, 'a_whole_new_tree');
        })(),
      ).rejects.toThrow();
    });

    it('should serialize sibling path data to a buffer and be able to deserialize it back', async () => {
      const db = await AztecLmdbStore.openTmp();
      const tree = await createDb(db, pedersen, 'test', 10);
      await appendLeaves(tree, values.slice(0, 1));

      const siblingPath = await tree.getSiblingPath(0n, true);
      const buf = siblingPath.toBuffer();
      const recovered = SiblingPath.fromBuffer(buf);
      expect(recovered).toEqual(siblingPath);
      const deserialized = SiblingPath.deserialize(buf);
      expect(deserialized.elem).toEqual(siblingPath);
      expect(deserialized.adv).toBe(4 + 10 * 32);

      const dummyData = Buffer.alloc(23, 1);
      const paddedBuf = Buffer.concat([dummyData, buf]);
      const recovered2 = SiblingPath.fromBuffer(paddedBuf, 23);
      expect(recovered2).toEqual(siblingPath);
      const deserialized2 = SiblingPath.deserialize(buf);
      expect(deserialized2.elem).toEqual(siblingPath);
      expect(deserialized2.adv).toBe(4 + 10 * 32);
    });
  });
};
