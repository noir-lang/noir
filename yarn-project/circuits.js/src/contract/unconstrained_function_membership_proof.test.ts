import { type ContractArtifact, type FunctionArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { type ContractClass } from '@aztec/types/contracts';

import { getTestContractArtifact } from '../tests/fixtures.js';
import { getContractClassFromArtifact } from './contract_class.js';
import { type ContractClassIdPreimage } from './contract_class_id.js';
import {
  createUnconstrainedFunctionMembershipProof,
  isValidUnconstrainedFunctionMembershipProof,
} from './unconstrained_function_membership_proof.js';

describe('unconstrained_function_membership_proof', () => {
  let artifact: ContractArtifact;
  let contractClass: ContractClass & ContractClassIdPreimage;
  let unconstrainedFunction: FunctionArtifact;
  let vkHash: Fr;
  let selector: FunctionSelector;

  beforeEach(() => {
    artifact = getTestContractArtifact();
    contractClass = getContractClassFromArtifact(artifact);
    unconstrainedFunction = artifact.functions.findLast(fn => fn.functionType === FunctionType.UNCONSTRAINED)!;
    selector = FunctionSelector.fromNameAndParameters(unconstrainedFunction);
  });

  const isUnconstrained = (fn: { functionType: FunctionType }) => fn.functionType === FunctionType.UNCONSTRAINED;

  it('computes and verifies a proof', () => {
    expect(unconstrainedFunction).toBeDefined();
    const proof = createUnconstrainedFunctionMembershipProof(selector, artifact);
    const fn = { ...unconstrainedFunction, ...proof, selector };
    expect(isValidUnconstrainedFunctionMembershipProof(fn, contractClass)).toBeTruthy();
  });

  it('handles a contract with a single function', () => {
    // Remove all unconstrained functions from the contract but one
    const unconstrainedFns = artifact.functions.filter(isUnconstrained);
    artifact.functions = artifact.functions.filter(fn => !isUnconstrained(fn) || fn === unconstrainedFns[0]);
    expect(artifact.functions.filter(isUnconstrained).length).toBe(1);

    const unconstrainedFunction = unconstrainedFns[0];
    const proof = createUnconstrainedFunctionMembershipProof(selector, artifact);
    expect(proof.artifactTreeSiblingPath.length).toBe(0);

    const fn = { ...unconstrainedFunction, ...proof, selector };
    const contractClass = getContractClassFromArtifact(artifact);
    expect(isValidUnconstrainedFunctionMembershipProof(fn, contractClass)).toBeTruthy();
  });

  test.each(['artifactTreeSiblingPath', 'artifactMetadataHash', 'functionMetadataHash'] as const)(
    'fails proof if %s is mangled',
    field => {
      const proof = createUnconstrainedFunctionMembershipProof(selector, artifact);
      const original = proof[field];
      const mangled = Array.isArray(original) ? [Fr.random(), ...original.slice(1)] : Fr.random();
      const wrong = { ...proof, [field]: mangled };
      const fn = { ...unconstrainedFunction, ...wrong, selector, vkHash };
      expect(isValidUnconstrainedFunctionMembershipProof(fn, contractClass)).toBeFalsy();
    },
  );
});
