import {
  AztecAddress,
  EthAddress,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';
import { ContractArtifact } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';

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
  initArgs: any[] = [],
  publicKey: Point = Point.ZERO,
  portalContractAddress = EthAddress.ZERO,
): ProtocolContract {
  // TODO(@spalladino): This computes the contract class from the artifact twice.
  const contractClass = getContractClassFromArtifact(artifact);
  const instance = getContractInstanceFromDeployParams(
    artifact,
    initArgs,
    new Fr(salt),
    publicKey,
    portalContractAddress,
  );
  return {
    instance,
    contractClass,
    artifact,
    address: instance.address,
  };
}
