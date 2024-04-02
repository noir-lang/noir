import {
  type AztecAddress,
  EthAddress,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { type ContractClassWithId, type ContractInstanceWithAddress } from '@aztec/types/contracts';

/** Represents a canonical contract in the protocol. */
export interface ProtocolContract {
  /** Canonical deployed instance. */
  instance: ContractInstanceWithAddress;
  /** Contract class of this contract. */
  contractClass: ContractClassWithId;
  /** Complete contract artifact. */
  artifact: ContractArtifact;
  /** Deployment address for the canonical instance.  */
  address: AztecAddress;
}

/** Returns the canonical deployment a given artifact. */
export function getCanonicalProtocolContract(
  artifact: ContractArtifact,
  salt: Fr | number | bigint,
  constructorArgs: any[] = [],
  publicKey: Point = Point.ZERO,
  portalAddress = EthAddress.ZERO,
): ProtocolContract {
  // TODO(@spalladino): This computes the contract class from the artifact twice.
  const contractClass = getContractClassFromArtifact(artifact);
  const instance = getContractInstanceFromDeployParams(artifact, {
    constructorArgs,
    salt: new Fr(salt),
    publicKey,
    portalAddress,
  });
  return {
    instance,
    contractClass,
    artifact,
    address: instance.address,
  };
}
