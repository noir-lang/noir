import {
  Fr,
  NullifierLeaf,
  NullifierLeafPreimage,
  PublicDataTreeLeaf,
  PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { Hasher } from '@aztec/types/interfaces';
import { SiblingPath } from '@aztec/types/membership';

import { INITIAL_LEAF, MerkleTree, Pedersen, loadTree, newTree } from '../../index.js';
import { treeTestSuite } from '../../test/test_suite.js';
import { StandardIndexedTreeWithAppend } from './standard_indexed_tree_with_append.js';

class NullifierTree extends StandardIndexedTreeWithAppend {
  constructor(store: AztecKVStore, hasher: Hasher, name: string, depth: number, size: bigint = 0n, root?: Buffer) {
    super(store, hasher, name, depth, size, NullifierLeafPreimage, NullifierLeaf, root);
  }
}

class PublicDataTree extends StandardIndexedTreeWithAppend {
  constructor(store: AztecKVStore, hasher: Hasher, name: string, depth: number, size: bigint = 0n, root?: Buffer) {
    super(store, hasher, name, depth, size, PublicDataTreeLeafPreimage, PublicDataTreeLeaf, root);
  }
}

const createDb = async (store: AztecKVStore, hasher: Hasher, name: string, depth: number, prefilledSize = 1) => {
  return await newTree(NullifierTree, store, hasher, name, depth, prefilledSize);
};

const createFromName = async (store: AztecKVStore, hasher: Hasher, name: string) => {
  return await loadTree(NullifierTree, store, hasher, name);
};

const createNullifierTreeLeafHashInputs = (value: number, nextIndex: number, nextValue: number) => {
  return new NullifierLeafPreimage(new Fr(value), new Fr(nextValue), BigInt(nextIndex)).toHashInputs();
};

const createPublicDataTreeLeaf = (slot: number, value: number) => {
  return new PublicDataTreeLeaf(new Fr(slot), new Fr(value));
};

const createPublicDataTreeLeafHashInputs = (slot: number, value: number, nextIndex: number, nextSlot: number) => {
  return new PublicDataTreeLeafPreimage(
    new Fr(slot),
    new Fr(value),
    new Fr(nextSlot),
    BigInt(nextIndex),
  ).toHashInputs();
};

const verifyCommittedState = async <N extends number>(
  tree: MerkleTree,
  root: Buffer,
  siblingPathIndex: bigint,
  emptySiblingPath: SiblingPath<N>,
) => {
  expect(tree.getRoot(false)).toEqual(root);
  expect(tree.getNumLeaves(false)).toEqual(1n);
  expect(await tree.getSiblingPath(siblingPathIndex, false)).toEqual(emptySiblingPath);
};

const TEST_TREE_DEPTH = 3;

treeTestSuite('StandardIndexedTree', createDb, createFromName);

describe('StandardIndexedTreeSpecific', () => {
  let pedersen: Pedersen;

  beforeEach(() => {
    pedersen = new Pedersen();
  });

  it('produces the correct roots and sibling paths', async () => {
    // Create a depth-3 indexed merkle tree
    const db = await AztecLmdbStore.openTmp();
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

    const initialLeafHash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 0, 0));
    const level1ZeroHash = pedersen.hash(INITIAL_LEAF, INITIAL_LEAF);
    const level2ZeroHash = pedersen.hash(level1ZeroHash, level1ZeroHash);

    let index0Hash = initialLeafHash;
    // Each element is named by the level followed by the index on that level. E.g. e10 -> level 1, index 0, e21 -> level 2, index 1
    let e10 = pedersen.hash(index0Hash, INITIAL_LEAF);
    let e20 = pedersen.hash(e10, level1ZeroHash);

    const initialE20 = e20; // Kept for calculating committed state later
    const initialE10 = e10;

    let root = pedersen.hash(e20, level2ZeroHash);
    const initialRoot = root;

    const emptySiblingPath = new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, level2ZeroHash]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(1n);
    expect(await tree.getSiblingPath(0n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, level2ZeroHash]),
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
    index0Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 1, 30));
    let index1Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(30, 0, 0));
    e10 = pedersen.hash(index0Hash, index1Hash);
    e20 = pedersen.hash(e10, level1ZeroHash);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(30n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(2n);
    expect(await tree.getSiblingPath(1n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index0Hash, level1ZeroHash, level2ZeroHash]),
    );

    // ensure the committed state is correct
    const initialSiblingPath = new SiblingPath(TEST_TREE_DEPTH, [initialLeafHash, level1ZeroHash, level2ZeroHash]);
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
    index0Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 2, 10));
    let index2Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(10, 1, 30));
    e10 = pedersen.hash(index0Hash, index1Hash);
    let e11 = pedersen.hash(index2Hash, INITIAL_LEAF);
    e20 = pedersen.hash(e10, e11);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(10n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(3n);
    expect(await tree.getSiblingPath(2n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e10, level2ZeroHash]),
    );

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      2n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, initialE10, level2ZeroHash]),
    );

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0.
     */
    e10 = pedersen.hash(index0Hash, index1Hash);
    index2Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(10, 3, 20));
    const index3Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(20, 1, 30));
    e11 = pedersen.hash(index2Hash, index3Hash);
    e20 = pedersen.hash(e10, e11);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(20n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(4n);
    expect(await tree.getSiblingPath(3n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index2Hash, e10, level2ZeroHash]),
    );

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      3n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, initialE10, level2ZeroHash]),
    );

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       50      0       0       0
     *  nextIdx   2       4       3       1        0       0       0       0
     *  nextVal   10      50      20      30       0       0       0       0.
     */
    index1Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(30, 4, 50));
    const index4Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(50, 0, 0));
    e10 = pedersen.hash(index0Hash, index1Hash);
    e20 = pedersen.hash(e10, e11);
    const e12 = pedersen.hash(index4Hash, INITIAL_LEAF);
    const e21 = pedersen.hash(e12, level1ZeroHash);
    root = pedersen.hash(e20, e21);

    await tree.appendLeaves([toBufferBE(50n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(5n);

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      4n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, initialE20]),
    );

    // check all uncommitted hash paths
    expect(await tree.getSiblingPath(0n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index1Hash, e11, e21]));
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index0Hash, e11, e21]));
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index3Hash, e10, e21]));
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index2Hash, e10, e21]));
    expect(await tree.getSiblingPath(4n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, e20]),
    );
    expect(await tree.getSiblingPath(5n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index4Hash, level1ZeroHash, e20]),
    );
    expect(await tree.getSiblingPath(6n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e12, e20]));
    expect(await tree.getSiblingPath(7n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e12, e20]));

    // check all committed hash paths
    expect(await tree.getSiblingPath(0n, false)).toEqual(emptySiblingPath);
    expect(await tree.getSiblingPath(1n, false)).toEqual(initialSiblingPath);
    expect(await tree.getSiblingPath(2n, false)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, initialE10, level2ZeroHash]),
    );
    expect(await tree.getSiblingPath(3n, false)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, initialE10, level2ZeroHash]),
    );
    const e2SiblingPath = new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, initialE20]);
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
    const tree = await createDb(await AztecLmdbStore.openTmp(), pedersen, 'test', 3);

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
    const initialLeafHash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 0, 0));
    const level1ZeroHash = pedersen.hash(INITIAL_LEAF, INITIAL_LEAF);
    const level2ZeroHash = pedersen.hash(level1ZeroHash, level1ZeroHash);
    let index0Hash = initialLeafHash;

    let e10 = pedersen.hash(index0Hash, INITIAL_LEAF);
    let e20 = pedersen.hash(e10, level1ZeroHash);

    const inite10 = e10;
    const inite20 = e20;

    let root = pedersen.hash(e20, level2ZeroHash);
    const initialRoot = root;

    const emptySiblingPath = new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, level2ZeroHash]);
    const initialSiblingPath = new SiblingPath(TEST_TREE_DEPTH, [initialLeafHash, level1ZeroHash, level2ZeroHash]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(1n);
    expect(await tree.getSiblingPath(0n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, level2ZeroHash]),
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
    index0Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 1, 30));
    let index1Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(30, 0, 0));
    e10 = pedersen.hash(index0Hash, index1Hash);
    e20 = pedersen.hash(e10, level1ZeroHash);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(30n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(2n);
    expect(await tree.getSiblingPath(1n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index0Hash, level1ZeroHash, level2ZeroHash]),
    );

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
    index0Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(0, 2, 10));
    let index2Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(10, 1, 30));
    e10 = pedersen.hash(index0Hash, index1Hash);
    let e11 = pedersen.hash(index2Hash, INITIAL_LEAF);
    e20 = pedersen.hash(e10, e11);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(10n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(3n);
    expect(await tree.getSiblingPath(2n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e10, level2ZeroHash]),
    );

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      2n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, inite10, level2ZeroHash]),
    );

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0.
     */
    e10 = pedersen.hash(index0Hash, index1Hash);
    index2Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(10, 3, 20));
    const index3Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(20, 1, 30));
    e11 = pedersen.hash(index2Hash, index3Hash);
    e20 = pedersen.hash(e10, e11);
    root = pedersen.hash(e20, level2ZeroHash);

    await tree.appendLeaves([toBufferBE(20n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(4n);
    expect(await tree.getSiblingPath(3n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index2Hash, e10, level2ZeroHash]),
    );

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      3n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, inite10, level2ZeroHash]),
    );

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
     *  nextVal   10      50      20      30       0       0       0       0.
     */
    index1Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(30, 6, 50));
    const index6Hash = pedersen.hashInputs(createNullifierTreeLeafHashInputs(50, 0, 0));
    e10 = pedersen.hash(index0Hash, index1Hash);
    e20 = pedersen.hash(e10, e11);
    const e13 = pedersen.hash(index6Hash, INITIAL_LEAF);
    const e21 = pedersen.hash(level1ZeroHash, e13);
    root = pedersen.hash(e20, e21);

    await tree.appendLeaves([toBufferBE(50n, 32)]);

    expect(tree.getRoot(true)).toEqual(root);
    expect(tree.getNumLeaves(true)).toEqual(7n);

    // ensure the committed state is correct
    await verifyCommittedState(
      tree,
      initialRoot,
      6n,
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, inite20]),
    );

    // // check all uncommitted hash paths
    expect(await tree.getSiblingPath(0n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index1Hash, e11, e21]));
    expect(await tree.getSiblingPath(1n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index0Hash, e11, e21]));
    expect(await tree.getSiblingPath(2n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index3Hash, e10, e21]));
    expect(await tree.getSiblingPath(3n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [index2Hash, e10, e21]));
    expect(await tree.getSiblingPath(4n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e13, e20]));
    expect(await tree.getSiblingPath(5n, true)).toEqual(new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, e13, e20]));
    expect(await tree.getSiblingPath(6n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, e20]),
    );
    expect(await tree.getSiblingPath(7n, true)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [index6Hash, level1ZeroHash, e20]),
    );

    // check all committed hash paths
    expect(await tree.getSiblingPath(0n, false)).toEqual(emptySiblingPath);
    expect(await tree.getSiblingPath(1n, false)).toEqual(initialSiblingPath);
    expect(await tree.getSiblingPath(2n, false)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, inite10, level2ZeroHash]),
    );
    expect(await tree.getSiblingPath(3n, false)).toEqual(
      new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, inite10, level2ZeroHash]),
    );
    const e2SiblingPath = new SiblingPath(TEST_TREE_DEPTH, [INITIAL_LEAF, level1ZeroHash, inite20]);
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

  // For varying orders of insertions assert the local batch insertion generator creates the correct proofs
  it.each([
    // These are arbitrary but it needs to be higher than the constant `INITIAL_NULLIFIER_TREE_SIZE` and `KERNEL_NEW_NULLIFIERS_LENGTH * 2`
    [[1003, 1002, 1001, 1000, 0, 0, 0, 0]],
    [[1003, 1004, 1005, 1006, 0, 0, 0, 0]],
    [[1234, 1098, 0, 0, 99999, 1096, 1054, 0]],
    [[1970, 1980, 1040, 0, 99999, 1880, 100001, 9000000]],
  ] as const)('performs nullifier tree batch insertion correctly', async nullifiers => {
    const leaves = nullifiers.map(i => toBufferBE(BigInt(i), 32));

    const TREE_HEIGHT = 16; // originally from NULLIFIER_TREE_HEIGHT
    const INITIAL_TREE_SIZE = 8; // originally from INITIAL_NULLIFIER_TREE_SIZE
    const SUBTREE_HEIGHT = 5; // originally from BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT

    // Create a depth-3 indexed merkle tree
    const appendTree = await createDb(await AztecLmdbStore.openTmp(), pedersen, 'test', TREE_HEIGHT, INITIAL_TREE_SIZE);
    const insertTree = await createDb(await AztecLmdbStore.openTmp(), pedersen, 'test', TREE_HEIGHT, INITIAL_TREE_SIZE);

    await appendTree.appendLeaves(leaves);
    await insertTree.batchInsert(leaves, SUBTREE_HEIGHT);

    const expectedRoot = appendTree.getRoot(true);
    const actualRoot = insertTree.getRoot(true);
    expect(actualRoot).toEqual(expectedRoot);
  });

  it('should be able to find indexes of leaves', async () => {
    const db = await AztecLmdbStore.openTmp();
    const tree = await createDb(db, pedersen, 'test', 3);
    const values = [Buffer.alloc(32, 1), Buffer.alloc(32, 2)];

    await tree.appendLeaves([values[0]]);

    expect(tree.findLeafIndex(values[0], true)).toBeDefined();
    expect(tree.findLeafIndex(values[0], false)).toBe(undefined);
    expect(tree.findLeafIndex(values[1], true)).toBe(undefined);

    await tree.commit();

    expect(tree.findLeafIndex(values[0], false)).toBeDefined();
  });

  describe('Updatable leaves', () => {
    it('should be able to upsert leaves', async () => {
      // Create a depth-3 indexed merkle tree
      const db = await AztecLmdbStore.openTmp();
      const tree = await newTree(PublicDataTree, db, pedersen, 'test', 3, 1);

      /**
       * Initial state:
       *
       *  index     0       1       2       3        4       5       6       7
       *  ---------------------------------------------------------------------
       *  slot      0       0       0       0        0       0       0       0
       *  value     0       0       0       0        0       0       0       0
       *  nextIdx   0       0       0       0        0       0       0       0
       *  nextSlot  0       0       0       0        0       0       0       0.
       */

      const EMPTY_LEAF = toBufferBE(0n, 32);
      const initialLeafHash = pedersen.hashInputs(createPublicDataTreeLeafHashInputs(0, 0, 0, 0));
      const level1ZeroHash = pedersen.hash(EMPTY_LEAF, EMPTY_LEAF);
      const level2ZeroHash = pedersen.hash(level1ZeroHash, level1ZeroHash);
      let index0Hash = initialLeafHash;

      let e10 = pedersen.hash(index0Hash, EMPTY_LEAF);
      let e20 = pedersen.hash(e10, level1ZeroHash);

      const inite10 = e10;

      let root = pedersen.hash(e20, level2ZeroHash);
      const initialRoot = root;

      const emptySiblingPath = new SiblingPath(TEST_TREE_DEPTH, [EMPTY_LEAF, level1ZeroHash, level2ZeroHash]);
      const initialSiblingPath = new SiblingPath(TEST_TREE_DEPTH, [initialLeafHash, level1ZeroHash, level2ZeroHash]);

      expect(tree.getRoot(true)).toEqual(root);
      expect(tree.getNumLeaves(true)).toEqual(1n);
      expect(await tree.getSiblingPath(0n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [EMPTY_LEAF, level1ZeroHash, level2ZeroHash]),
      );

      await verifyCommittedState(tree, initialRoot, 0n, emptySiblingPath);

      /**
       * Add new value 30:5:
       *
       *  index     0       1       2       3        4       5       6       7
       *  ---------------------------------------------------------------------
       *  slot      0       30      0       0        0       0       0       0
       *  value     0       5       0       0        0       0       0       0
       *  nextIdx   1       0       0       0        0       0       0       0
       *  nextSlot  30      0       0       0        0       0       0       0.
       */
      index0Hash = pedersen.hashInputs(createPublicDataTreeLeafHashInputs(0, 0, 1, 30));
      let index1Hash = pedersen.hashInputs(createPublicDataTreeLeafHashInputs(30, 5, 0, 0));
      e10 = pedersen.hash(index0Hash, index1Hash);
      e20 = pedersen.hash(e10, level1ZeroHash);
      root = pedersen.hash(e20, level2ZeroHash);

      await tree.appendLeaves([createPublicDataTreeLeaf(30, 5).toBuffer()]);

      expect(tree.getRoot(true)).toEqual(root);
      expect(tree.getNumLeaves(true)).toEqual(2n);
      expect(await tree.getSiblingPath(1n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [index0Hash, level1ZeroHash, level2ZeroHash]),
      );

      // ensure the committed state is correct
      await verifyCommittedState(tree, initialRoot, 1n, initialSiblingPath);

      /**
       *  Update the value of 30 to 10:
       *
       *  index     0       1       2       3        4       5       6       7
       *  ---------------------------------------------------------------------
       *  slot      0       30      0       0        0       0       0       0
       *  value     0       10      0       0        0       0       0       0
       *  nextIdx   1       0       0       0        0       0       0       0
       *  nextSlot  30      0       0       0        0       0       0       0.
       */
      index1Hash = pedersen.hashInputs(createPublicDataTreeLeafHashInputs(30, 10, 0, 0));
      e10 = pedersen.hash(index0Hash, index1Hash);
      e20 = pedersen.hash(e10, level1ZeroHash);
      root = pedersen.hash(e20, level2ZeroHash);

      await tree.appendLeaves([createPublicDataTreeLeaf(30, 10).toBuffer()]);

      expect(tree.getRoot(true)).toEqual(root);
      expect(tree.getNumLeaves(true)).toEqual(3n);
      expect(await tree.getSiblingPath(2n, true)).toEqual(
        new SiblingPath(TEST_TREE_DEPTH, [EMPTY_LEAF, e10, level2ZeroHash]),
      );

      // ensure the committed state is correct
      await verifyCommittedState(
        tree,
        initialRoot,
        2n,
        new SiblingPath(TEST_TREE_DEPTH, [EMPTY_LEAF, inite10, level2ZeroHash]),
      );
    });

    it.each([
      [[createPublicDataTreeLeaf(1, 10)], [createPublicDataTreeLeaf(1, 20)]],
      [[createPublicDataTreeLeaf(1, 10)], [createPublicDataTreeLeaf(1, 20), createPublicDataTreeLeaf(2, 5)]],
      [
        [createPublicDataTreeLeaf(1, 10), createPublicDataTreeLeaf(2, 10)],
        [createPublicDataTreeLeaf(1, 20), createPublicDataTreeLeaf(10, 50), createPublicDataTreeLeaf(2, 5)],
      ],
    ] as const)('performs batch upsert correctly', async (initialState, batch) => {
      const TREE_HEIGHT = 16;
      const INITIAL_TREE_SIZE = 8;
      const SUBTREE_HEIGHT = 5;

      const db = await AztecLmdbStore.openTmp();
      const appendTree = await newTree(PublicDataTree, db, pedersen, 'test', TREE_HEIGHT, INITIAL_TREE_SIZE);
      const insertTree = await newTree(PublicDataTree, db, pedersen, 'test', TREE_HEIGHT, INITIAL_TREE_SIZE);

      await appendTree.appendLeaves(initialState.map(leaf => leaf.toBuffer()));
      await insertTree.appendLeaves(initialState.map(leaf => leaf.toBuffer()));

      await appendTree.appendLeaves(batch.map(leaf => leaf.toBuffer()));
      await insertTree.batchInsert(
        batch.map(leaf => leaf.toBuffer()),
        SUBTREE_HEIGHT,
      );

      const expectedRoot = appendTree.getRoot(true);
      const actualRoot = insertTree.getRoot(true);
      expect(actualRoot).toEqual(expectedRoot);
    });
  });
});
