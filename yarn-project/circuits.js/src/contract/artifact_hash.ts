import { ContractArtifact, FunctionArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { numToUInt8 } from '@aztec/foundation/serialize';

import { MerkleTree } from '../merkle/merkle_tree.js';
import { MerkleTreeCalculator } from '../merkle/merkle_tree_calculator.js';

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
export function computeArtifactHash(artifact: ContractArtifact): Fr {
  const privateFunctionRoot = computeArtifactFunctionTreeRoot(artifact, FunctionType.SECRET);
  const unconstrainedFunctionRoot = computeArtifactFunctionTreeRoot(artifact, FunctionType.UNCONSTRAINED);
  const metadataHash = computeArtifactMetadataHash(artifact);
  const preimage = [numToUInt8(VERSION), privateFunctionRoot, unconstrainedFunctionRoot, metadataHash];
  // TODO(@spalladino) Reducing sha256 to a field may have security implications. Validate this with crypto team.
  return Fr.fromBufferReduce(sha256(Buffer.concat(preimage)));
}

export function computeArtifactMetadataHash(artifact: ContractArtifact) {
  const metadata = { name: artifact.name, events: artifact.events }; // TODO(@spalladino): Should we use the sorted event selectors instead? They'd need to be unique for that.
  return sha256(Buffer.from(JSON.stringify(metadata), 'utf-8'));
}

export function computeArtifactFunctionTreeRoot(artifact: ContractArtifact, fnType: FunctionType) {
  return computeArtifactFunctionTree(artifact, fnType)?.root ?? Fr.ZERO.toBuffer();
}

export function computeArtifactFunctionTree(artifact: ContractArtifact, fnType: FunctionType): MerkleTree | undefined {
  const leaves = computeFunctionLeaves(artifact, fnType);
  // TODO(@spalladino) Consider implementing a null-object for empty trees
  if (leaves.length === 0) {
    return undefined;
  }
  const height = Math.ceil(Math.log2(leaves.length));
  const calculator = new MerkleTreeCalculator(height, Buffer.alloc(32), (l, r) => sha256(Buffer.concat([l, r])));
  return calculator.computeTree(leaves);
}

function computeFunctionLeaves(artifact: ContractArtifact, fnType: FunctionType) {
  return artifact.functions
    .filter(f => f.functionType === fnType)
    .map(f => ({ ...f, selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters) }))
    .sort((a, b) => a.selector.value - b.selector.value)
    .map(computeFunctionArtifactHash);
}

export function computeFunctionArtifactHash(fn: FunctionArtifact & { selector?: FunctionSelector }): Buffer {
  const selector =
    (fn as { selector: FunctionSelector }).selector ?? FunctionSelector.fromNameAndParameters(fn.name, fn.parameters);
  const bytecodeHash = sha256(Buffer.from(fn.bytecode, 'hex'));
  const metadata = JSON.stringify(fn.returnTypes);
  const metadataHash = sha256(Buffer.from(metadata, 'utf8'));
  return sha256(Buffer.concat([numToUInt8(VERSION), selector.toBuffer(), metadataHash, bytecodeHash]));
}
