import { type ContractArtifact, type FunctionArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { type ContractClass } from '@aztec/types/contracts';

import { getBenchmarkContractArtifact } from '../tests/fixtures.js';
import { computeVerificationKeyHash, getContractClassFromArtifact } from './contract_class.js';
import { type ContractClassIdPreimage } from './contract_class_id.js';
import {
  createPrivateFunctionMembershipProof,
  isValidPrivateFunctionMembershipProof,
} from './private_function_membership_proof.js';

describe('private_function_membership_proof', () => {
  let artifact: ContractArtifact;
  let contractClass: ContractClass & ContractClassIdPreimage;
  let privateFunction: FunctionArtifact;
  let vkHash: Fr;
  let selector: FunctionSelector;

  beforeAll(() => {
    artifact = getBenchmarkContractArtifact();
    contractClass = getContractClassFromArtifact(artifact);
    privateFunction = artifact.functions.findLast(fn => fn.functionType === FunctionType.SECRET)!;
    vkHash = computeVerificationKeyHash(privateFunction.verificationKey!);
    selector = FunctionSelector.fromNameAndParameters(privateFunction);
  });

  it('computes and verifies a proof', () => {
    const proof = createPrivateFunctionMembershipProof(selector, artifact);
    const fn = { ...privateFunction, ...proof, selector, vkHash };
    expect(isValidPrivateFunctionMembershipProof(fn, contractClass)).toBeTruthy();
  });

  test.each([
    'artifactTreeSiblingPath',
    'artifactMetadataHash',
    'functionMetadataHash',
    'unconstrainedFunctionsArtifactTreeRoot',
    'privateFunctionTreeSiblingPath',
  ] as const)('fails proof if %s is mangled', field => {
    const proof = createPrivateFunctionMembershipProof(selector, artifact);
    const original = proof[field];
    const mangled = Array.isArray(original) ? [Fr.random(), ...original.slice(1)] : Fr.random();
    const wrong = { ...proof, [field]: mangled };
    const fn = { ...privateFunction, ...wrong, selector, vkHash };
    expect(isValidPrivateFunctionMembershipProof(fn, contractClass)).toBeFalsy();
  });
});
