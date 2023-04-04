import times from 'lodash.times';
import { FUNCTION_TREE_HEIGHT } from '../index.js';
import { fr } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { computeFunctionTree, getDummyPreviousKernelData } from './kernel.js';

describe('abis wasm bindings', () => {
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.new();
  });

  it.skip('gets dummy kernel data', async () => {
    await expect(getDummyPreviousKernelData(wasm)).resolves.toBeDefined();
  });

  it('computes function tree', async () => {
    const numLeaves = 4;
    const leaves = times(numLeaves, i => fr(i));
    const tree = await computeFunctionTree(wasm, leaves);
    expect(tree).toHaveLength(2 ** (FUNCTION_TREE_HEIGHT + 1) - 1);
    expect(tree.slice(0, numLeaves)).toEqual(leaves);
  });
});
