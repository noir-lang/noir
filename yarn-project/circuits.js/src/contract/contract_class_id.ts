import { pedersenHash, sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { numToUInt8 } from '@aztec/foundation/serialize';
import { ContractClass, PrivateFunction, PublicFunction } from '@aztec/types/contracts';

import { MerkleTreeCalculator } from '../abis/merkle_tree_calculator.js';
import { FUNCTION_TREE_HEIGHT, GeneratorIndex } from '../constants.gen.js';

/**
 * Returns the id of a contract class computed as its hash.
 *
 * ```
 * version = 1
 * private_function_leaves = private_functions.map(fn => pedersen([fn.function_selector as Field, fn.vk_hash], GENERATOR__FUNCTION_LEAF))
 * private_functions_root = merkleize(private_function_leaves)
 * bytecode_commitment = calculate_commitment(packed_bytecode)
 * contract_class_id = pedersen([version, artifact_hash, private_functions_root, bytecode_commitment], GENERATOR__CLASS_IDENTIFIER)
 * ```
 * @param contractClass - Contract class.
 * @returns The identifier.
 */
export function getContractClassId(contractClass: ContractClass): Fr {
  const privateFunctionsRoot = getPrivateFunctionsRoot(contractClass.privateFunctions);
  const publicFunctionsRoot = getPublicFunctionsRoot(contractClass.publicFunctions); // This should be removed once we drop public functions as first class citizens in the protocol
  const bytecodeCommitment = getBytecodeCommitment(contractClass.packedBytecode);
  return Fr.fromBuffer(
    pedersenHash(
      [
        numToUInt8(contractClass.version),
        contractClass.artifactHash.toBuffer(),
        privateFunctionsRoot.toBuffer(),
        publicFunctionsRoot.toBuffer(),
        bytecodeCommitment.toBuffer(),
      ],
      GeneratorIndex.CONTRACT_LEAF, // TODO(@spalladino): Review all generator indices in this file
    ),
  );
}

// TODO(@spalladino): Replace with actual implementation
function getBytecodeCommitment(bytecode: Buffer) {
  return Fr.fromBufferReduce(sha256(bytecode));
}

// Memoize the merkle tree calculators to avoid re-computing the zero-hash for each level in each call
let privateFunctionTreeCalculator: MerkleTreeCalculator | undefined;
let publicFunctionTreeCalculator: MerkleTreeCalculator | undefined;

const PRIVATE_FUNCTION_SIZE = 2;
const PUBLIC_FUNCTION_SIZE = 2;

function getPrivateFunctionsRoot(fns: PrivateFunction[]): Fr {
  const privateFunctionLeaves = fns.map(fn =>
    pedersenHash(
      [fn.selector, fn.vkHash].map(x => x.toBuffer()),
      GeneratorIndex.FUNCTION_LEAF,
    ),
  );
  if (!privateFunctionTreeCalculator) {
    const functionTreeZeroLeaf = pedersenHash(new Array(PRIVATE_FUNCTION_SIZE).fill(Buffer.alloc(32)));
    privateFunctionTreeCalculator = new MerkleTreeCalculator(FUNCTION_TREE_HEIGHT, functionTreeZeroLeaf);
  }
  return Fr.fromBuffer(privateFunctionTreeCalculator.computeTreeRoot(privateFunctionLeaves));
}

function getPublicFunctionsRoot(fns: PublicFunction[]): Fr {
  const publicFunctionLeaves = fns.map(fn =>
    pedersenHash(
      [fn.selector, getBytecodeCommitment(fn.bytecode)].map(x => x.toBuffer()),
      GeneratorIndex.FUNCTION_LEAF,
    ),
  );
  if (!publicFunctionTreeCalculator) {
    const functionTreeZeroLeaf = pedersenHash(new Array(PUBLIC_FUNCTION_SIZE).fill(Buffer.alloc(32)));
    publicFunctionTreeCalculator = new MerkleTreeCalculator(FUNCTION_TREE_HEIGHT, functionTreeZeroLeaf);
  }
  return Fr.fromBuffer(publicFunctionTreeCalculator.computeTreeRoot(publicFunctionLeaves));
}
