import { ContractArtifact } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { ContractInstance, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { getContractClassFromArtifact } from '../contract/contract_class.js';
import { computeContractClassId } from '../contract/contract_class_id.js';
import { PublicKey } from '../types/public_key.js';
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
    constructorName?: string;
    constructorArgs?: any[];
    salt?: Fr;
    publicKey?: PublicKey;
    portalAddress?: EthAddress;
  },
): ContractInstanceWithAddress {
  const args = opts.constructorArgs ?? [];
  const salt = opts.salt ?? Fr.random();
  const publicKey = opts.publicKey ?? Point.ZERO;
  const portalContractAddress = opts.portalAddress ?? EthAddress.ZERO;
  const constructorName = opts.constructorName ?? 'constructor';

  const constructorArtifact = artifact.functions.find(fn => fn.name === constructorName);
  if (!constructorArtifact) {
    throw new Error(`Cannot find constructor with name ${constructorName} in the artifact.`);
  }
  if (!constructorArtifact.verificationKey) {
    throw new Error('Missing verification key for the constructor.');
  }

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
    version: 1,
  };

  return { ...instance, address: computeContractAddressFromInstance(instance) };
}
