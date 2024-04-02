import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type PrivateFunction } from '@aztec/types/contracts';

import { FUNCTION_TREE_HEIGHT, GeneratorIndex } from '../constants.gen.js';
import { type MerkleTree, MerkleTreeCalculator } from '../merkle/index.js';

// Memoize the merkle tree calculators to avoid re-computing the zero-hash for each level in each call
let privateFunctionTreeCalculator: MerkleTreeCalculator | undefined;

const PRIVATE_FUNCTION_SIZE = 2;

/** Returns a Merkle tree for the set of private functions in a contract. */
export function computePrivateFunctionsTree(fns: PrivateFunction[]): MerkleTree {
  return getPrivateFunctionTreeCalculator().computeTree(computePrivateFunctionLeaves(fns));
}

/** Returns the Merkle tree root for the set of private functions in a contract. */
export function computePrivateFunctionsRoot(fns: PrivateFunction[]): Fr {
  return Fr.fromBuffer(getPrivateFunctionTreeCalculator().computeTreeRoot(computePrivateFunctionLeaves(fns)));
}

function computePrivateFunctionLeaves(fns: PrivateFunction[]): Buffer[] {
  const leaves = [...fns].sort((a, b) => a.selector.value - b.selector.value);
  return leaves.map(computePrivateFunctionLeaf);
}

/** Returns the leaf for a given private function. */
export function computePrivateFunctionLeaf(fn: PrivateFunction): Buffer {
  return pedersenHash([fn.selector, fn.vkHash], GeneratorIndex.FUNCTION_LEAF).toBuffer();
}

function getPrivateFunctionTreeCalculator(): MerkleTreeCalculator {
  if (!privateFunctionTreeCalculator) {
    const functionTreeZeroLeaf = pedersenHash(new Array(PRIVATE_FUNCTION_SIZE).fill(Buffer.alloc(32))).toBuffer();
    privateFunctionTreeCalculator = new MerkleTreeCalculator(FUNCTION_TREE_HEIGHT, functionTreeZeroLeaf);
  }
  return privateFunctionTreeCalculator;
}
