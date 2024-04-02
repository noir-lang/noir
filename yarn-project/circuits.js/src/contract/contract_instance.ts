import { type ContractArtifact, type FunctionArtifact, getDefaultInitializer } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { type ContractInstance, type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { getContractClassFromArtifact } from '../contract/contract_class.js';
import { computeContractClassId } from '../contract/contract_class_id.js';
import { type PublicKey } from '../types/public_key.js';
import {
  computeContractAddressFromInstance,
  computeInitializationHash,
  computePublicKeysHash,
} from './contract_address.js';

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
    publicKey?: PublicKey;
    portalAddress?: EthAddress;
    deployer?: AztecAddress;
  },
): ContractInstanceWithAddress {
  const args = opts.constructorArgs ?? [];
  const salt = opts.salt ?? Fr.random();
  const publicKey = opts.publicKey ?? Point.ZERO;
  const portalContractAddress = opts.portalAddress ?? EthAddress.ZERO;
  const constructorArtifact = getConstructorArtifact(artifact, opts.constructorArtifact);
  const deployer = opts.deployer ?? AztecAddress.ZERO;

  const contractClass = getContractClassFromArtifact(artifact);
  const contractClassId = computeContractClassId(contractClass);
  const initializationHash = computeInitializationHash(constructorArtifact, args);
  const publicKeysHash = computePublicKeysHash(publicKey);

  const instance: ContractInstance = {
    contractClassId,
    initializationHash,
    portalContractAddress,
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
