import { default as levelup } from 'levelup';
import { Hasher, INITIAL_LEAF, MerkleTree, Pedersen, SiblingPath } from '../index.js';
import { StandardIndexedTree } from './standard_indexed_tree.js';
import { treeTestSuite } from '../test/test_suite.js';
import { toBufferBE } from '@aztec/foundation';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { createMemDown } from '../test/utils/create_mem_down.js';
import { newTree } from '../new_tree.js';
import { loadTree } from '../load_tree.js';

const createDb = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string, depth: number) => {
  return await newTree(StandardIndexedTree, levelUp, hasher, name, depth);
};

const createFromName = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string) => {
  return await loadTree(StandardIndexedTree, levelUp, hasher, name);
};

const createIndexedTreeLeaf = (value: number, nextIndex: number, nextValue: number) => {
  return [toBufferBE(BigInt(value), 32), toBufferBE(BigInt(nextIndex), 32), toBufferBE(BigInt(nextValue), 32)];
};

const verifyCommittedState = async (
  tree: MerkleTree,
  root: Buffer,
  siblingPathIndex: bigint,
  emptySiblingPath: SiblingPath,
) => {
  expect(tree.getRoot(false)).toEqual(root);
  expect(tree.getNumLeaves(false)).toEqual(1n);
  expect(await tree.getSiblingPath(siblingPathIndex, false)).toEqual(emptySiblingPath);
};

treeTestSuite('StandardIndexedTree', createDb, createFromName);

describe('StandardIndexedTreeSpecific', () => {
  let wasm: BarretenbergWasm;
  let pedersen: Pedersen;

  beforeEach(async () => {
    wasm = await BarretenbergWasm.get();
    pedersen = new Pedersen(wasm);
  });

  it('produces the correct roots and sibling paths', async () => {
    // Create a depth-3 indexed merkle tree
    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', 3);

    /**
     * Initial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       0       0       0        0       0       0       0
     *  nextIdx   0       0       0       0        0       0       0       0
     *  nextVal   0       0       0       0        0       0       0       0.
     */

    const initialLeafHash = pedersen.compressInputs(createIndexedTreeLeaf(0, 0, 0));
    const level1ZeroHash = pedersen.compress(INITIAL_LEAF, INITIAL_LEAF);
    const level2ZeroHash = pedersen.compress(level1ZeroHash, level1ZeroHash);

    let index0Hash = initialLeafHash;
    // Each element is named by the level followed by the index on that level. E.g. e10 -> level 1, index 0, e21 -> level 2, index 1
    let e10 = pedersen.compress(index0Hash, INITIAL_LEAF);
    let e20 = pedersen.compress(e10, level1ZeroHash);

    const initialE20 = e20; // Kept for calculating committed state later
    const initialE10 = e10;

    let root = pedersen.compress(e20, level2ZeroHash);
    const initialRoot = root;

    const emptySiblingPath = new SiblingPath([INITIAL_LEAF, level1ZeroHash, level2ZeroHash]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(1n);
    expect(await tree.getSiblingPath(0n, true)).toEqual(
      new SiblingPath([INITIAL_LEAF, level1ZeroHash, level2ZeroHash]),
    );

    await verifyCommittedState(tree, initialRoot, 0n, emptySiblingPath);

    /**
     * Add new value 30:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   30      0       0       0        0       0       0       0.
     */
    index0Hash = pedersen.compressInputs(createIndexedTreeLeaf(0, 1, 30));
    let index1Hash = pedersen.compressInputs(createIndexedTreeLeaf(30, 0, 0));
    e10 = pedersen.compress(index0Hash, index1Hash);
    e20 = pedersen.compress(e10, level1ZeroHash);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(30n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(2n);
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath([index0Hash, level1ZeroHash, level2ZeroHash]));

    // ensure the committed state is correct
    const initialSiblingPath = new SiblingPath([initialLeafHash, level1ZeroHash, level2ZeroHash]);
    await verifyCommittedState(tree, initialRoot, 1n, initialSiblingPath);

    /**
     * Add new value 10:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      0        0       0       0       0
     *  nextIdx   2       0       1       0        0       0       0       0
     *  nextVal   10      0       30      0        0       0       0       0.
     */
    index0Hash = pedersen.compressInputs(createIndexedTreeLeaf(0, 2, 10));
    let index2Hash = pedersen.compressInputs(createIndexedTreeLeaf(10, 1, 30));
    e10 = pedersen.compress(index0Hash, index1Hash);
    let e11 = pedersen.compress(index2Hash, INITIAL_LEAF);
    e20 = pedersen.compress(e10, e11);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(10n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(3n);
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e10, level2ZeroHash]));

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 2n, new SiblingPath([INITIAL_LEAF, initialE10, level2ZeroHash]));

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0.
     */
    e10 = pedersen.compress(index0Hash, index1Hash);
    index2Hash = pedersen.compressInputs(createIndexedTreeLeaf(10, 3, 20));
    const index3Hash = pedersen.compressInputs(createIndexedTreeLeaf(20, 1, 30));
    e11 = pedersen.compress(index2Hash, index3Hash);
    e20 = pedersen.compress(e10, e11);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(20n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(4n);
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath([index2Hash, e10, level2ZeroHash]));

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 3n, new SiblingPath([INITIAL_LEAF, initialE10, level2ZeroHash]));

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       50      0       0       0
     *  nextIdx   2       4       3       1        0       0       0       0
     *  nextVal   10      50      20      30       0       0       0       0.
     */
    index1Hash = pedersen.compressInputs(createIndexedTreeLeaf(30, 4, 50));
    const index4Hash = pedersen.compressInputs(createIndexedTreeLeaf(50, 0, 0));
    e10 = pedersen.compress(index0Hash, index1Hash);
    e20 = pedersen.compress(e10, e11);
    const e12 = pedersen.compress(index4Hash, INITIAL_LEAF);
    const e21 = pedersen.compress(e12, level1ZeroHash);
    root = pedersen.compress(e20, e21);

    await tree.appendLeaves([toBufferBE(50n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(5n);

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 4n, new SiblingPath([INITIAL_LEAF, level1ZeroHash, initialE20]));

    // check all uncommitted hash paths
    expect(await tree.getSiblingPath(0n, true)).toEqual(new SiblingPath([index1Hash, e11, e21]));
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath([index0Hash, e11, e21]));
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath([index3Hash, e10, e21]));
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath([index2Hash, e10, e21]));
    expect(await tree.getSiblingPath(4n, true)).toEqual(new SiblingPath([INITIAL_LEAF, level1ZeroHash, e20]));
    expect(await tree.getSiblingPath(5n, true)).toEqual(new SiblingPath([index4Hash, level1ZeroHash, e20]));
    expect(await tree.getSiblingPath(6n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e12, e20]));
    expect(await tree.getSiblingPath(7n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e12, e20]));

    // check all committed hash paths
    expect(await tree.getSiblingPath(0n, false)).toEqual(emptySiblingPath);
    expect(await tree.getSiblingPath(1n, false)).toEqual(initialSiblingPath);
    expect(await tree.getSiblingPath(2n, false)).toEqual(new SiblingPath([INITIAL_LEAF, initialE10, level2ZeroHash]));
    expect(await tree.getSiblingPath(3n, false)).toEqual(new SiblingPath([INITIAL_LEAF, initialE10, level2ZeroHash]));
    const e2SiblingPath = new SiblingPath([INITIAL_LEAF, level1ZeroHash, initialE20]);
    expect(await tree.getSiblingPath(4n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(5n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(6n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(7n, false)).toEqual(e2SiblingPath);

    await tree.commit();
    // check all committed hash paths equal uncommitted hash paths
    for (let i = 0; i < 8; i++) {
      expect(await tree.getSiblingPath(BigInt(i), false)).toEqual(await tree.getSiblingPath(BigInt(i), true));
    }
  });

  it('Can append empty leaves and handle insertions', async () => {
    // Create a depth-3 indexed merkle tree
    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', 3);

    /**
     * Initial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       0       0       0        0       0       0       0
     *  nextIdx   0       0       0       0        0       0       0       0
     *  nextVal   0       0       0       0        0       0       0       0.
     */

    const INITIAL_LEAF = toBufferBE(0n, 32);
    const initialLeafHash = pedersen.compressInputs(createIndexedTreeLeaf(0, 0, 0));
    const level1ZeroHash = pedersen.compress(INITIAL_LEAF, INITIAL_LEAF);
    const level2ZeroHash = pedersen.compress(level1ZeroHash, level1ZeroHash);
    let index0Hash = initialLeafHash;

    let e10 = pedersen.compress(index0Hash, INITIAL_LEAF);
    let e20 = pedersen.compress(e10, level1ZeroHash);

    const inite10 = e10;
    const inite20 = e20;

    let root = pedersen.compress(e20, level2ZeroHash);
    const initialRoot = root;

    const emptySiblingPath = new SiblingPath([INITIAL_LEAF, level1ZeroHash, level2ZeroHash]);
    const initialSiblingPath = new SiblingPath([initialLeafHash, level1ZeroHash, level2ZeroHash]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(1n);
    expect(await tree.getSiblingPath(0n, true)).toEqual(
      new SiblingPath([INITIAL_LEAF, level1ZeroHash, level2ZeroHash]),
    );

    await verifyCommittedState(tree, initialRoot, 0n, emptySiblingPath);

    /**
     * Add new value 30:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   30      0       0       0        0       0       0       0.
     */
    index0Hash = pedersen.compressInputs(createIndexedTreeLeaf(0, 1, 30));
    let index1Hash = pedersen.compressInputs(createIndexedTreeLeaf(30, 0, 0));
    e10 = pedersen.compress(index0Hash, index1Hash);
    e20 = pedersen.compress(e10, level1ZeroHash);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(30n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(2n);
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath([index0Hash, level1ZeroHash, level2ZeroHash]));

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 1n, initialSiblingPath);

    /**
     * Add new value 10:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      0        0       0       0       0
     *  nextIdx   2       0       1       0        0       0       0       0
     *  nextVal   10      0       30      0        0       0       0       0.
     */
    index0Hash = pedersen.compressInputs(createIndexedTreeLeaf(0, 2, 10));
    let index2Hash = pedersen.compressInputs(createIndexedTreeLeaf(10, 1, 30));
    e10 = pedersen.compress(index0Hash, index1Hash);
    let e11 = pedersen.compress(index2Hash, INITIAL_LEAF);
    e20 = pedersen.compress(e10, e11);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(10n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(3n);
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e10, level2ZeroHash]));

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 2n, new SiblingPath([INITIAL_LEAF, inite10, level2ZeroHash]));

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0.
     */
    e10 = pedersen.compress(index0Hash, index1Hash);
    index2Hash = pedersen.compressInputs(createIndexedTreeLeaf(10, 3, 20));
    const index3Hash = pedersen.compressInputs(createIndexedTreeLeaf(20, 1, 30));
    e11 = pedersen.compress(index2Hash, index3Hash);
    e20 = pedersen.compress(e10, e11);
    root = pedersen.compress(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(20n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(4n);
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath([index2Hash, e10, level2ZeroHash]));

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 3n, new SiblingPath([INITIAL_LEAF, inite10, level2ZeroHash]));

    // Add 2 empty values
    const emptyLeaves = [toBufferBE(0n, 32), toBufferBE(0n, 32)];
    await tree.appendLeaves(emptyLeaves);

    // The root should be the same but the size should have increased
    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(6n);

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  --------------------------------------------------------------------
     *  val       0       30      10      20       0       0       50      0
     *  nextIdx   2       6       3       1        0       0       0       0
     *  nextVal   10      50      20      30       0       0       0       0
     */
    index1Hash = pedersen.compressInputs(createIndexedTreeLeaf(30, 6, 50));
    const index6Hash = pedersen.compressInputs(createIndexedTreeLeaf(50, 0, 0));
    e10 = pedersen.compress(index0Hash, index1Hash);
    e20 = pedersen.compress(e10, e11);
    const e13 = pedersen.compress(index6Hash, INITIAL_LEAF);
    const e21 = pedersen.compress(level1ZeroHash, e13);
    root = pedersen.compress(e20, e21);

    await tree.appendLeaves([toBufferBE(50n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(7n);

    // ensure the committed state is correct
    await verifyCommittedState(tree, initialRoot, 6n, new SiblingPath([INITIAL_LEAF, level1ZeroHash, inite20]));

    // // check all uncommitted hash paths
    expect(await tree.getSiblingPath(0n, true)).toEqual(new SiblingPath([index1Hash, e11, e21]));
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath([index0Hash, e11, e21]));
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath([index3Hash, e10, e21]));
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath([index2Hash, e10, e21]));
    expect(await tree.getSiblingPath(4n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e13, e20]));
    expect(await tree.getSiblingPath(5n, true)).toEqual(new SiblingPath([INITIAL_LEAF, e13, e20]));
    expect(await tree.getSiblingPath(6n, true)).toEqual(new SiblingPath([INITIAL_LEAF, level1ZeroHash, e20]));
    expect(await tree.getSiblingPath(7n, true)).toEqual(new SiblingPath([index6Hash, level1ZeroHash, e20]));

    // check all committed hash paths
    expect(await tree.getSiblingPath(0n, false)).toEqual(emptySiblingPath);
    expect(await tree.getSiblingPath(1n, false)).toEqual(initialSiblingPath);
    expect(await tree.getSiblingPath(2n, false)).toEqual(new SiblingPath([INITIAL_LEAF, inite10, level2ZeroHash]));
    expect(await tree.getSiblingPath(3n, false)).toEqual(new SiblingPath([INITIAL_LEAF, inite10, level2ZeroHash]));
    const e2SiblingPath = new SiblingPath([INITIAL_LEAF, level1ZeroHash, inite20]);
    expect(await tree.getSiblingPath(4n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(5n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(6n, false)).toEqual(e2SiblingPath);
    expect(await tree.getSiblingPath(7n, false)).toEqual(e2SiblingPath);

    await tree.commit();
    // check all committed hash paths equal uncommitted hash paths
    for (let i = 0; i < 8; i++) {
      expect(await tree.getSiblingPath(BigInt(i), false)).toEqual(await tree.getSiblingPath(BigInt(i), true));
    }
  });
});
