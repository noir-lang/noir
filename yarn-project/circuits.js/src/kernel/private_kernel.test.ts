import times from 'lodash.times';

import { computeFunctionTreeRoot } from '../abis/abis.js';
import { privateKernelDummyPreviousKernel } from '../cbind/circuits.gen.js';
import { FUNCTION_TREE_HEIGHT } from '../cbind/constants.gen.js';
import { fr } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { computeFunctionTree } from './private_kernel.js';

describe('kernel/private_kernel', () => {
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  it('gets dummy kernel data', () => {
    expect(privateKernelDummyPreviousKernel(wasm)).toBeDefined();
  });

  it('computes function tree', () => {
    const numLeaves = 4;
    const leaves = times(numLeaves, i => fr(i));
    const tree = computeFunctionTree(wasm, leaves);

    expect(tree).toHaveLength(2 ** (FUNCTION_TREE_HEIGHT + 1) - 1);
    expect(tree.slice(0, numLeaves)).toEqual(leaves);

    const root = tree[tree.length - 1];
    expect(root).toEqual(computeFunctionTreeRoot(wasm, leaves));
  });
});
