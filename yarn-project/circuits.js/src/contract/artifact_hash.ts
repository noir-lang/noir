import { type ContractArtifact, type FunctionArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr, reduceFn } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { numToUInt8 } from '@aztec/foundation/serialize';

import { type MerkleTree } from '../merkle/merkle_tree.js';
import { MerkleTreeCalculator } from '../merkle/merkle_tree_calculator.js';

const VERSION = 1;

// TODO(miranda): Artifact and artifact metadata hashes are currently the only SHAs not truncated by a byte.
// They are never recalculated in the circuit or L1 contract, but they are input to circuits, so perhaps modding here is preferable?
// TODO(@spalladino) Reducing sha256 to a field may have security implications. Validate this with crypto team.
const sha256Fr = reduceFn(sha256, Fr);

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
export function computeArtifactHash(
  artifact: ContractArtifact | { privateFunctionRoot: Fr; unconstrainedFunctionRoot: Fr; metadataHash: Fr },
): Fr {
  if ('privateFunctionRoot' in artifact && 'unconstrainedFunctionRoot' in artifact && 'metadataHash' in artifact) {
    const { privateFunctionRoot, unconstrainedFunctionRoot, metadataHash } = artifact;
    const preimage = [privateFunctionRoot, unconstrainedFunctionRoot, metadataHash].map(x => x.toBuffer());
    return sha256Fr(Buffer.concat([numToUInt8(VERSION), ...preimage]));
  }

  const preimage = computeArtifactHashPreimage(artifact);
  const artifactHash = computeArtifactHash(computeArtifactHashPreimage(artifact));
  getLogger().trace('Computed artifact hash', { artifactHash, ...preimage });
  return artifactHash;
}

export function computeArtifactHashPreimage(artifact: ContractArtifact) {
  const privateFunctionRoot = computeArtifactFunctionTreeRoot(artifact, FunctionType.SECRET);
  const unconstrainedFunctionRoot = computeArtifactFunctionTreeRoot(artifact, FunctionType.UNCONSTRAINED);
  const metadataHash = computeArtifactMetadataHash(artifact);
  return { privateFunctionRoot, unconstrainedFunctionRoot, metadataHash };
}

export function computeArtifactMetadataHash(artifact: ContractArtifact) {
  // TODO(@spalladino): Should we use the sorted event selectors instead? They'd need to be unique for that.
  const metadata = { name: artifact.name, events: artifact.events };
  return sha256Fr(Buffer.from(JSON.stringify(metadata), 'utf-8'));
}

export function computeArtifactFunctionTreeRoot(artifact: ContractArtifact, fnType: FunctionType) {
  const root = computeArtifactFunctionTree(artifact, fnType)?.root;
  return root ? Fr.fromBuffer(root) : Fr.ZERO;
}

export function computeArtifactFunctionTree(artifact: ContractArtifact, fnType: FunctionType): MerkleTree | undefined {
  const leaves = computeFunctionLeaves(artifact, fnType);
  // TODO(@spalladino) Consider implementing a null-object for empty trees
  if (leaves.length === 0) {
    return undefined;
  }
  const height = Math.ceil(Math.log2(leaves.length));
  const calculator = new MerkleTreeCalculator(height, Buffer.alloc(32), getArtifactMerkleTreeHasher());
  return calculator.computeTree(leaves.map(x => x.toBuffer()));
}

function computeFunctionLeaves(artifact: ContractArtifact, fnType: FunctionType) {
  return artifact.functions
    .filter(f => f.functionType === fnType)
    .map(f => ({ ...f, selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters) }))
    .sort((a, b) => a.selector.value - b.selector.value)
    .map(computeFunctionArtifactHash);
}

export function computeFunctionArtifactHash(
  fn:
    | FunctionArtifact
    | (Pick<FunctionArtifact, 'bytecode'> & { functionMetadataHash: Fr; selector: FunctionSelector }),
) {
  const selector = 'selector' in fn ? fn.selector : FunctionSelector.fromNameAndParameters(fn);
  const bytecodeHash = sha256Fr(fn.bytecode).toBuffer();
  const metadataHash = 'functionMetadataHash' in fn ? fn.functionMetadataHash : computeFunctionMetadataHash(fn);
  return sha256Fr(Buffer.concat([numToUInt8(VERSION), selector.toBuffer(), metadataHash.toBuffer(), bytecodeHash]));
}

export function computeFunctionMetadataHash(fn: FunctionArtifact) {
  return sha256Fr(Buffer.from(JSON.stringify(fn.returnTypes), 'utf8'));
}

function getLogger() {
  return createDebugLogger('aztec:circuits:artifact_hash');
}

export function getArtifactMerkleTreeHasher() {
  return (l: Buffer, r: Buffer) => sha256Fr(Buffer.concat([l, r])).toBuffer();
}
