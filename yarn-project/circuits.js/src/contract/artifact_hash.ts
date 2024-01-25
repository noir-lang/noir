import { ContractArtifact, FunctionArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { numToUInt8 } from '@aztec/foundation/serialize';

import { MerkleTreeCalculator } from '../abis/merkle_tree_calculator.js';

const VERSION = 1;

/**
 * Returns the artifact hash of a given compiled contract artifact.
 *
 * ```
 * private_functions_artifact_leaves = artifact.private_functions.map fn =>
 *   sha256(fn.selector, fn.metadata_hash, sha256(fn.bytecode))
 * private_functions_artifact_tree_root = merkleize(private_functions_artifact_leaves)
 *
 * unconstrained_functions_artifact_leaves = artifact.unconstrained_functions.map fn =>
 *   sha256(fn.selector, fn.metadata_hash, sha256(fn.bytecode))
 * unconstrained_functions_artifact_tree_root = merkleize(unconstrained_functions_artifact_leaves)
 *
 * version = 1
 * artifact_hash = sha256(
 *   version,
 *   private_functions_artifact_tree_root,
 *   unconstrained_functions_artifact_tree_root,
 *   artifact_metadata,
 * )
 * ```
 * @param artifact - Artifact to calculate the hash for.
 */
export function getArtifactHash(artifact: ContractArtifact): Fr {
  const privateFunctionRoot = getFunctionRoot(artifact, FunctionType.SECRET);
  const unconstrainedFunctionRoot = getFunctionRoot(artifact, FunctionType.OPEN);
  const metadataHash = getArtifactMetadataHash(artifact);
  const preimage = [numToUInt8(VERSION), privateFunctionRoot, unconstrainedFunctionRoot, metadataHash];
  return Fr.fromBufferReduce(sha256(Buffer.concat(preimage)));
}

function getArtifactMetadataHash(artifact: ContractArtifact) {
  const metadata = { name: artifact.name, events: artifact.events }; // TODO(@spalladino): Should we use the sorted event selectors instead? They'd need to be unique for that.
  return sha256(Buffer.from(JSON.stringify(metadata), 'utf-8'));
}

type FunctionArtifactWithSelector = FunctionArtifact & { selector: FunctionSelector };

function getFunctionRoot(artifact: ContractArtifact, fnType: FunctionType) {
  const leaves = getFunctionLeaves(artifact, fnType);
  const height = Math.ceil(Math.log2(leaves.length));
  const calculator = new MerkleTreeCalculator(height, Buffer.alloc(32), (l, r) => sha256(Buffer.concat([l, r])));
  return calculator.computeTreeRoot(leaves);
}

function getFunctionLeaves(artifact: ContractArtifact, fnType: FunctionType) {
  return artifact.functions
    .filter(f => f.functionType === fnType)
    .map(f => ({ ...f, selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters) }))
    .sort((a, b) => a.selector.value - b.selector.value)
    .map(getFunctionArtifactHash);
}

function getFunctionArtifactHash(fn: FunctionArtifactWithSelector): Buffer {
  const bytecodeHash = sha256(Buffer.from(fn.bytecode, 'hex'));
  const metadata = JSON.stringify(fn.returnTypes);
  const metadataHash = sha256(Buffer.from(metadata, 'utf8'));
  return sha256(Buffer.concat([numToUInt8(VERSION), fn.selector.toBuffer(), metadataHash, bytecodeHash]));
}
