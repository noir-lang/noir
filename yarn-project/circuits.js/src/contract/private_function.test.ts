import { Fr } from '@aztec/foundation/fields';
import { PrivateFunction } from '@aztec/types/contracts';

import { fr, makeSelector } from '../tests/factories.js';
import { computePrivateFunctionsRoot, computePrivateFunctionsTree } from './private_function.js';

describe('PrivateFunction', () => {
  const privateFunctions: PrivateFunction[] = [
    { selector: makeSelector(1), vkHash: fr(2), isInternal: false },
    { selector: makeSelector(3), vkHash: fr(4), isInternal: false },
  ];

  it('computes merkle tree', () => {
    const tree = computePrivateFunctionsTree(privateFunctions);
    expect(tree.nodes.map(node => node.toString())).toMatchSnapshot();
  });

  it('computes merkle tree root', () => {
    const root = computePrivateFunctionsRoot(privateFunctions);
    expect(root.toString()).toMatchSnapshot();
  });

  it('tree and root methods agree', () => {
    const tree = computePrivateFunctionsTree(privateFunctions);
    const root = computePrivateFunctionsRoot(privateFunctions);
    expect(Fr.fromBuffer(tree.root).equals(root)).toBe(true);
  });

  it('sorts functions before computing tree', () => {
    const root = computePrivateFunctionsRoot(privateFunctions);
    const rootReversed = computePrivateFunctionsRoot([...privateFunctions].reverse());
    expect(root.equals(rootReversed)).toBe(true);
  });
});
