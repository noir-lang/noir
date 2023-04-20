import { default as levelup } from 'levelup';
import { Hasher } from '../hasher.js';
import { treeTestSuite } from '../test/test_suite.js';
import { SparseTree } from './sparse_tree.js';
import { standardBasedTreeTestSuite } from '../test/standard_based_test_suite.js';
import { createMemDown } from '../test/utils/create_mem_down.js';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { Pedersen } from '../pedersen.js';
import { randomBytes } from 'crypto';
import { INITIAL_LEAF, SiblingPath } from '../index.js';
import { UpdateOnlyTree } from '../interfaces/update_only_tree.js';
import { newTree } from '../new_tree.js';
import { loadTree } from '../load_tree.js';

const createDb = async (
  levelUp: levelup.LevelUp,
  hasher: Hasher,
  name: string,
  depth: number,
): Promise<UpdateOnlyTree> => {
  return await newTree(SparseTree, levelUp, hasher, name, depth);
};

const createFromName = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string): Promise<UpdateOnlyTree> => {
  return await loadTree(SparseTree, levelUp, hasher, name);
};

treeTestSuite('SparseTree', createDb, createFromName);
standardBasedTreeTestSuite('SparseTree', createDb);

describe('SparseTreeSpecific', () => {
  let wasm: BarretenbergWasm;
  let pedersen: Pedersen;

  beforeEach(async () => {
    wasm = await BarretenbergWasm.get();
    pedersen = new Pedersen(wasm);
  });

  it('throws when index is bigger than (2^DEPTH - 1) ', async () => {
    const db = levelup(createMemDown());
    const depth = 32;
    const tree = await createDb(db, pedersen, 'test', depth);

    const index = 2n ** BigInt(depth);
    await expect(tree.updateLeaf(Buffer.alloc(32), index)).rejects.toThrow();
  });

  it('updating non-empty leaf does not change tree size', async () => {
    const depth = 32;
    const maxIndex = 2 ** depth - 1;

    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', depth);

    const randomIndex = BigInt(Math.floor(Math.random() * maxIndex));
    expect(tree.getNumLeaves(false)).toEqual(0n);

    // Insert a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);

    // Update a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);
  });

  it('deleting leaf decrements tree size', async () => {
    const depth = 254;
    const maxIndex = 2 ** depth - 1;

    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', depth);

    const randomIndex = BigInt(Math.floor(Math.random() * maxIndex));
    expect(tree.getNumLeaves(false)).toEqual(0n);

    // Insert a leaf
    await tree.updateLeaf(randomBytes(32), randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(1n);

    // Delete a leaf
    await tree.updateLeaf(INITIAL_LEAF, randomIndex);
    expect(tree.getNumLeaves(true)).toEqual(0n);
  });

  it('should have correct root and sibling path after in a "non-append-only" way', async () => {
    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', 3);

    const level2ZeroHash = pedersen.compress(INITIAL_LEAF, INITIAL_LEAF);
    const level1ZeroHash = pedersen.compress(level2ZeroHash, level2ZeroHash);

    expect(tree.getNumLeaves(false)).toEqual(0n);
    expect(tree.getRoot(false)).toEqual(pedersen.compress(level1ZeroHash, level1ZeroHash));

    // Insert leaf at index 3
    let level1LeftHash: Buffer;
    const leafAtIndex3 = randomBytes(32);
    {
      await tree.updateLeaf(leafAtIndex3, 3n);
      expect(tree.getNumLeaves(true)).toEqual(1n);
      const level2Hash = pedersen.compress(INITIAL_LEAF, leafAtIndex3);
      level1LeftHash = pedersen.compress(level2ZeroHash, level2Hash);
      const root = pedersen.compress(level1LeftHash, level1ZeroHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath([INITIAL_LEAF, level2ZeroHash, level1ZeroHash]),
      );
    }

    // Insert leaf at index 6
    let level1RightHash: Buffer;
    {
      const leafAtIndex6 = randomBytes(32);
      await tree.updateLeaf(leafAtIndex6, 6n);
      expect(tree.getNumLeaves(true)).toEqual(2n);
      const level2Hash = pedersen.compress(leafAtIndex6, INITIAL_LEAF);
      level1RightHash = pedersen.compress(level2ZeroHash, level2Hash);
      const root = pedersen.compress(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(6n, true)).toEqual(
        new SiblingPath([INITIAL_LEAF, level2ZeroHash, level1LeftHash]),
      );
    }

    // Insert leaf at index 2
    const leafAtIndex2 = randomBytes(32);
    {
      await tree.updateLeaf(leafAtIndex2, 2n);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      const level2Hash = pedersen.compress(leafAtIndex2, leafAtIndex3);
      level1LeftHash = pedersen.compress(level2ZeroHash, level2Hash);
      const root = pedersen.compress(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(2n, true)).toEqual(
        new SiblingPath([leafAtIndex3, level2ZeroHash, level1RightHash]),
      );
    }

    // Updating leaf at index 3
    {
      const updatedLeafAtIndex3 = randomBytes(32);
      await tree.updateLeaf(updatedLeafAtIndex3, 3n);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      const level2Hash = pedersen.compress(leafAtIndex2, updatedLeafAtIndex3);
      level1LeftHash = pedersen.compress(level2ZeroHash, level2Hash);
      const root = pedersen.compress(level1LeftHash, level1RightHash);
      expect(tree.getRoot(true)).toEqual(root);
      expect(await tree.getSiblingPath(3n, true)).toEqual(
        new SiblingPath([leafAtIndex2, level2ZeroHash, level1RightHash]),
      );
    }
  });

  it.skip('measures time of inserting 1000 leaves at random positions for depth 254', async () => {
    const depth = 254;
    const maxIndex = 2 ** depth - 1;

    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', depth);

    const leaves = Array.from({ length: 1000 }).map(() => randomBytes(32));
    const indices = Array.from({ length: 1000 }).map(() => BigInt(Math.floor(Math.random() * maxIndex)));

    const start = Date.now();
    await Promise.all(leaves.map((leaf, i) => tree.updateLeaf(leaf, indices[i])));
    const end = Date.now();
    console.log(`Inserting 1000 leaves at random positions for depth 254 took ${end - start}ms`);
  }, 300_000);
});
