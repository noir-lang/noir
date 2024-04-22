import { type ContractArtifact, type FunctionArtifact, getDefaultInitializer } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstance, type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { getContractClassFromArtifact } from '../contract/contract_class.js';
import { computeContractClassId } from '../contract/contract_class_id.js';
import { computeContractAddressFromInstance, computeInitializationHash } from './contract_address.js';

/**
 * Generates a Contract Instance from the deployment params.
 * @param artifact - The account contract build artifact.
 * @param opts - Options for the deployment.
 * @returns - The contract instance
 */
export function getContractInstanceFromDeployParams(
  artifact: ContractArtifact,
  opts: {
    constructorArtifact?: FunctionArtifact | string;
    constructorArgs?: any[];
    salt?: Fr;
    publicKeysHash?: Fr;
    deployer?: AztecAddress;
  },
): ContractInstanceWithAddress {
  const args = opts.constructorArgs ?? [];
  const salt = opts.salt ?? Fr.random();
  const constructorArtifact = getConstructorArtifact(artifact, opts.constructorArtifact);
  const deployer = opts.deployer ?? AztecAddress.ZERO;

  const contractClass = getContractClassFromArtifact(artifact);
  const contractClassId = computeContractClassId(contractClass);
  const initializationHash = computeInitializationHash(constructorArtifact, args);
  const publicKeysHash = opts.publicKeysHash ?? Fr.ZERO;

  const instance: ContractInstance = {
    contractClassId,
    initializationHash,
    publicKeysHash,
    salt,
    deployer,
    version: 1,
  };

  return { ...instance, address: computeContractAddressFromInstance(instance) };
}

function getConstructorArtifact(
  artifact: ContractArtifact,
  requestedConstructorArtifact: FunctionArtifact | string | undefined,
): FunctionArtifact | undefined {
  if (typeof requestedConstructorArtifact === 'string') {
    const found = artifact.functions.find(fn => fn.name === requestedConstructorArtifact);
    if (!found) {
      throw new Error(`No constructor found with name ${requestedConstructorArtifact}`);
    }
    return found;
  }
  return requestedConstructorArtifact ?? getDefaultInitializer(artifact);
}
