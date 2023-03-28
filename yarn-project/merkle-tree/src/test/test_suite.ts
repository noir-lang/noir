import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { default as levelup } from 'levelup';
import { default as memdown } from 'memdown';
import { Hasher, MerkleTree, Pedersen, SiblingPath } from '../index.js';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
export const createMemDown = () => memdown();

const expectSameTrees = async (tree1: MerkleTree, tree2: MerkleTree) => {
  const size = tree1.getNumLeaves();
  expect(size).toBe(tree2.getNumLeaves());
  expect(tree1.getRoot().toString('hex')).toBe(tree2.getRoot().toString('hex'));

  for (let i = 0; i < size; ++i) {
    const siblingPath1 = await tree1.getSiblingPath(BigInt(i));
    const siblingPath2 = await tree2.getSiblingPath(BigInt(i));
    expect(siblingPath2).toStrictEqual(siblingPath1);
  }
};

export const merkleTreeTestSuite = (
  testName: string,
  createDb: (levelup: levelup.LevelUp, hasher: Hasher, name: string, depth: number) => Promise<MerkleTree>,
  createFromName: (levelup: levelup.LevelUp, hasher: Hasher, name: string) => Promise<MerkleTree>,
) => {
  describe(testName, () => {
    const values: Buffer[] = [];
    let wasm: BarretenbergWasm;
    let pedersen: Pedersen;

    beforeAll(() => {
      for (let i = 0; i < 32; ++i) {
        const v = Buffer.alloc(32, i + 1);
        v.writeUInt32BE(i, 28);
        values[i] = v;
      }
    });

    beforeEach(async () => {
      wasm = await BarretenbergWasm.new();
      pedersen = new Pedersen(wasm);
    });

    it('should revert changes on rollback', async () => {
      const levelDownEmpty = createMemDown();
      const dbEmpty = levelup(levelDownEmpty);
      const emptyTree = await createDb(dbEmpty, pedersen, 'test', 10);

      const levelDown = createMemDown();
      const db = levelup(levelDown);
      const tree = await createDb(db, pedersen, 'test2', 10);
      await tree.appendLeaves(values.slice(0, 4));

      const firstRoot = tree.getRoot();

      expect(firstRoot).not.toEqual(emptyTree.getRoot());

      await tree.rollback();

      await expectSameTrees(tree, emptyTree);

      // append the leaves again
      await tree.appendLeaves(values.slice(0, 4));

      expect(tree.getRoot()).toEqual(firstRoot);

      expect(firstRoot).not.toEqual(emptyTree.getRoot());

      await tree.rollback();

      await expectSameTrees(tree, emptyTree);
    });

    it('should not revert changes after commit', async () => {
      const levelDownEmpty = createMemDown();
      const dbEmpty = levelup(levelDownEmpty);
      const emptyTree = await createDb(dbEmpty, pedersen, 'test', 10);

      const levelDown = createMemDown();
      const db = levelup(levelDown);
      const tree = await createDb(db, pedersen, 'test2', 10);
      await tree.appendLeaves(values.slice(0, 4));

      expect(tree.getRoot()).not.toEqual(emptyTree.getRoot());

      await tree.commit();
      await tree.rollback();

      expect(tree.getRoot()).not.toEqual(emptyTree.getRoot());
    });

    it('should be able to restore from previous committed data', async () => {
      const levelDown = createMemDown();
      const db = levelup(levelDown);
      const tree = await createDb(db, pedersen, 'test', 10);
      await tree.appendLeaves(values.slice(0, 4));
      await tree.commit();

      const db2 = levelup(levelDown);
      const tree2 = await createFromName(db2, pedersen, 'test');

      expect(tree.getRoot()).toEqual(tree2.getRoot());
      for (let i = 0; i < 4; ++i) {
        expect(await tree.getSiblingPath(BigInt(i))).toEqual(await tree2.getSiblingPath(BigInt(i)));
      }
    });

    it('should throw an error if previous data does not exist for the given name', async () => {
      const db = levelup(createMemDown());
      await expect(
        (async () => {
          await createFromName(db, pedersen, 'a_whole_new_tree');
        })(),
      ).rejects.toThrow();
    });

    it('should serialize sibling path data to a buffer and be able to deserialize it back', async () => {
      const db = levelup(createMemDown());
      const tree = await createDb(db, pedersen, 'test', 10);
      await tree.appendLeaves(values.slice(0, 1));

      const siblingPath = await tree.getSiblingPath(0n);
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
