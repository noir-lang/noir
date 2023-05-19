import times from 'lodash.times';
import { fr } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { computeFunctionTreeRoot } from '../abis/abis.js';
import { privateKernelDummyPreviousKernel } from '../cbind/circuits.gen.js';
import { computeFunctionTree } from './private_kernel.js';
import { FUNCTION_TREE_HEIGHT } from '../structs/index.js';

describe('kernel/private_kernel', () => {
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  it('gets dummy kernel data', async () => {
    await expect(privateKernelDummyPreviousKernel(wasm)).resolves.toBeDefined();
  });

  it('computes function tree', async () => {
    const numLeaves = 4;
    const leaves = times(numLeaves, i => fr(i));
    const tree = await computeFunctionTree(wasm, leaves);

    expect(tree).toHaveLength(2 ** (FUNCTION_TREE_HEIGHT + 1) - 1);
    expect(tree.slice(0, numLeaves)).toEqual(leaves);

    const root = tree[tree.length - 1];
    expect(root).toEqual(await computeFunctionTreeRoot(wasm, leaves));
  });
});
