import { type ContractArtifact, type FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import {
  type ContractClassPublic,
  type UnconstrainedFunctionMembershipProof,
  type UnconstrainedFunctionWithMembershipProof,
} from '@aztec/types/contracts';

import { computeRootFromSiblingPath } from '../merkle/index.js';
import {
  computeArtifactFunctionTree,
  computeArtifactHash,
  computeArtifactHashPreimage,
  computeFunctionArtifactHash,
  computeFunctionMetadataHash,
  getArtifactMerkleTreeHasher,
} from './artifact_hash.js';

/**
 * Creates a membership proof for an unconstrained function in a contract class, to be verified via `isValidUnconstrainedFunctionMembershipProof`.
 * @param selector - Selector of the function to create the proof for.
 * @param artifact - Artifact of the contract class where the function is defined.
 */
export function createUnconstrainedFunctionMembershipProof(
  selector: FunctionSelector,
  artifact: ContractArtifact,
): UnconstrainedFunctionMembershipProof {
  const log = createDebugLogger('aztec:circuits:function_membership_proof');

  // Locate function artifact
  const fn = artifact.functions.find(fn => selector.equals(fn));
  if (!fn) {
    throw new Error(`Function with selector ${selector.toString()} not found`);
  } else if (fn.functionType !== FunctionType.UNCONSTRAINED) {
    throw new Error(`Function ${fn.name} with selector ${selector.toString()} is not unconstrained`);
  }

  // Compute preimage for the artifact hash
  const { privateFunctionRoot: privateFunctionsArtifactTreeRoot, metadataHash: artifactMetadataHash } =
    computeArtifactHashPreimage(artifact);

  // Compute the sibling path for the "artifact tree"
  const functionMetadataHash = computeFunctionMetadataHash(fn);
  const functionArtifactHash = computeFunctionArtifactHash({ ...fn, functionMetadataHash });
  const artifactTree = computeArtifactFunctionTree(artifact, FunctionType.UNCONSTRAINED)!;
  const artifactTreeLeafIndex = artifactTree.getIndex(functionArtifactHash.toBuffer());
  const artifactTreeSiblingPath = artifactTree.getSiblingPath(artifactTreeLeafIndex).map(Fr.fromBuffer);

  log.trace(`Computed proof for unconstrained function with selector ${selector.toString()}`, {
    functionArtifactHash,
    functionMetadataHash,
    artifactMetadataHash,
    artifactFunctionTreeSiblingPath: artifactTreeSiblingPath.map(fr => fr.toString()).join(','),
    privateFunctionsArtifactTreeRoot,
  });

  return {
    artifactTreeSiblingPath,
    artifactTreeLeafIndex,
    artifactMetadataHash,
    functionMetadataHash,
    privateFunctionsArtifactTreeRoot,
  };
}

/**
 * Verifies that an unconstrained function with a membership proof as emitted by the ClassRegisterer contract is valid,
 * as defined in the yellow paper at contract-deployment/classes:
 *
 * ```
 * // Load contract class from local db
 * contract_class = db.get_contract_class(contract_class_id)
 *
 * // Compute artifact leaf and assert it belongs to the artifact
 * artifact_function_leaf = sha256(selector, metadata_hash, sha256(bytecode))
 * computed_artifact_unconstrained_function_tree_root = compute_root(artifact_function_leaf, artifact_function_tree_sibling_path, artifact_function_tree_leaf_index)
 * computed_artifact_hash = sha256(private_functions_artifact_tree_root, computed_artifact_unconstrained_function_tree_root, artifact_metadata_hash)
 * assert computed_artifact_hash == contract_class.artifact_hash
 * ```
 * @param fn - Function to check membership proof for.
 * @param contractClass - In which contract class the function is expected to be.
 */
export function isValidUnconstrainedFunctionMembershipProof(
  fn: UnconstrainedFunctionWithMembershipProof,
  contractClass: Pick<ContractClassPublic, 'artifactHash'>,
) {
  const log = createDebugLogger('aztec:circuits:function_membership_proof');

  const functionArtifactHash = computeFunctionArtifactHash(fn);
  const computedArtifactFunctionTreeRoot = Fr.fromBuffer(
    computeRootFromSiblingPath(
      functionArtifactHash.toBuffer(),
      fn.artifactTreeSiblingPath.map(fr => fr.toBuffer()),
      fn.artifactTreeLeafIndex,
      getArtifactMerkleTreeHasher(),
    ),
  );
  const computedArtifactHash = computeArtifactHash({
    privateFunctionRoot: fn.privateFunctionsArtifactTreeRoot,
    unconstrainedFunctionRoot: computedArtifactFunctionTreeRoot,
    metadataHash: fn.artifactMetadataHash,
  });
  if (!contractClass.artifactHash.equals(computedArtifactHash)) {
    log.trace(`Artifact hash mismatch`, {
      expected: contractClass.artifactHash,
      computedArtifactHash,
      computedFunctionArtifactHash: functionArtifactHash,
      computedArtifactFunctionTreeRoot,
      privateFunctionsArtifactTreeRoot: fn.privateFunctionsArtifactTreeRoot,
      metadataHash: fn.artifactMetadataHash,
      artifactFunctionTreeSiblingPath: fn.artifactTreeSiblingPath.map(fr => fr.toString()).join(','),
    });
    return false;
  }

  return true;
}
