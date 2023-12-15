import {
  computeCompleteAddress,
  computeFunctionTreeRoot,
  computeVarArgsHash,
  hashConstructor,
} from '@aztec/circuits.js/abis';
import { ContractArtifact, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';

import { DeploymentInfo, Fr, FunctionData, PublicKey } from '../index.js';
import { generateFunctionLeaves, hashVKStr, isConstructor } from './contract_tree/contract_tree.js';

/**
 * Generates the deployment info for a contract
 * @param artifact - The account contract build artifact.
 * @param args - The args to the account contract constructor
 * @param contractAddressSalt - The salt to be used in the contract address derivation
 * @param publicKey - The account public key
 * @returns - The contract deployment info
 */
export function getContractDeploymentInfo(
  artifact: ContractArtifact,
  args: any[],
  contractAddressSalt: Fr,
  publicKey: PublicKey,
): DeploymentInfo {
  const constructorArtifact = artifact.functions.find(isConstructor);
  if (!constructorArtifact) {
    throw new Error('Cannot find constructor in the artifact.');
  }
  if (!constructorArtifact.verificationKey) {
    throw new Error('Missing verification key for the constructor.');
  }

  const vkHash = hashVKStr(constructorArtifact.verificationKey);
  const constructorVkHash = Fr.fromBuffer(vkHash);
  const functions = artifact.functions.map(f => ({
    ...f,
    selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters),
  }));
  const leaves = generateFunctionLeaves(functions);
  const functionTreeRoot = computeFunctionTreeRoot(leaves);
  const functionData = FunctionData.fromAbi(constructorArtifact);
  const flatArgs = encodeArguments(constructorArtifact, args);
  const argsHash = computeVarArgsHash(flatArgs);
  const constructorHash = hashConstructor(functionData, argsHash, constructorVkHash.toBuffer());

  const completeAddress = computeCompleteAddress(publicKey, contractAddressSalt, functionTreeRoot, constructorHash);

  return {
    completeAddress,
    constructorHash,
    constructorVkHash,
    functionTreeRoot,
  };
}
