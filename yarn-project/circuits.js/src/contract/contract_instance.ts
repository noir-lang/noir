import { ContractArtifact } from '@aztec/foundation/abi';
import { ContractInstance, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { EthAddress, Fr, Point, PublicKey, computeContractClassId, getContractClassFromArtifact } from '../index.js';
import {
  computeContractAddressFromInstance,
  computeInitializationHash,
  computePublicKeysHash,
} from './contract_address.js';
import { isConstructor } from './contract_tree/contract_tree.js';

/**
 * Generates a Contract Instance from the deployment params.
 * @param artifact - The account contract build artifact.
 * @param args - The args to the account contract constructor
 * @param contractAddressSalt - The salt to be used in the contract address derivation
 * @param publicKey - The account public key
 * @param portalContractAddress - The portal contract address
 * @returns - The contract instance
 */
export function getContractInstanceFromDeployParams(
  artifact: ContractArtifact,
  args: any[] = [],
  contractAddressSalt: Fr = Fr.random(),
  publicKey: PublicKey = Point.ZERO,
  portalContractAddress: EthAddress = EthAddress.ZERO,
): ContractInstanceWithAddress {
  const constructorArtifact = artifact.functions.find(isConstructor);
  if (!constructorArtifact) {
    throw new Error('Cannot find constructor in the artifact.');
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
    salt: contractAddressSalt,
    version: 1,
  };

  return { ...instance, address: computeContractAddressFromInstance(instance) };
}
