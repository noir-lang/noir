import { CircuitsWasm } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { IWasmModule } from '@aztec/foundation/wasm';
import { Hasher } from '@aztec/types';

import { default as levelup } from 'levelup';

import { loadTree } from '../load_tree.js';
import { newTree } from '../new_tree.js';
import { standardBasedTreeTestSuite } from '../test/standard_based_test_suite.js';
import { treeTestSuite } from '../test/test_suite.js';
import { createMemDown } from '../test/utils/create_mem_down.js';
import { PedersenWithCounter } from '../test/utils/pedersen_with_counter.js';
import { INITIAL_LEAF } from '../tree_base.js';
import { StandardTree } from './standard_tree.js';

const createDb = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string, depth: number) => {
  return await newTree(StandardTree, levelUp, hasher, name, depth);
};

const createFromName = async (levelUp: levelup.LevelUp, hasher: Hasher, name: string) => {
  return await loadTree(StandardTree, levelUp, hasher, name);
};

treeTestSuite('StandardTree', createDb, createFromName);
standardBasedTreeTestSuite('StandardTree', createDb);

describe('StandardTree_batchAppend', () => {
  let wasm: IWasmModule;
  let pedersen: PedersenWithCounter;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
    pedersen = new PedersenWithCounter(wasm);
  });

  afterEach(() => {
    pedersen.resetCounter();
  });

  it('correctly computes root when batch appending and calls compress function expected num times', async () => {
    const db = levelup(createMemDown());
    const tree = await createDb(db, pedersen, 'test', 3);
    const leaves = Array.from({ length: 5 }, _ => randomBytes(32));

    pedersen.resetCounter();
    await tree.appendLeaves(leaves);

    // We append 5 leaves so to update values we do the following hashing on each level:
    //              level2Node0           level2Node1           level2Node2
    // LEVEL2: [newLeaf0, newLeaf1], [newLeaf2, newLeaf3], [newLeaf4, INITIAL_LEAF].
    //                    level1Node0                 level1Node1
    // LEVEL1:    [level2Node0, level2Node1], [level2Node2, level2ZeroHash].
    //                                       ROOT
    // LEVEL0:                   [level1Node0, level1Node1].
    const level2NumHashing = 3;
    const level1NumHashing = 2;
    const level0NumHashing = 1;
    const expectedNumHashing = level2NumHashing + level1NumHashing + level0NumHashing;

    expect(pedersen.compressCounter).toEqual(expectedNumHashing);

    const level2Node0 = pedersen.compress(leaves[0], leaves[1]);
    const level2Node1 = pedersen.compress(leaves[2], leaves[3]);
    const level2Node2 = pedersen.compress(leaves[4], INITIAL_LEAF);

    const level2ZeroHash = pedersen.compress(INITIAL_LEAF, INITIAL_LEAF);

    const level1Node0 = pedersen.compress(level2Node0, level2Node1);
    const level1Node1 = pedersen.compress(level2Node2, level2ZeroHash);

    const root = pedersen.compress(level1Node0, level1Node1);

    expect(tree.getRoot(true)).toEqual(root);
  });
});
